//! LAN collaboration: one person hosts, another joins by IP address, and
//! from then on every completed edit (not every per-frame drag tick) is
//! broadcast as a full project snapshot to whoever's connected. There's no
//! merge logic — the last snapshot to arrive simply replaces the local
//! project (last-writer-wins), which is the simplest thing that behaves
//! predictably for two people taking turns on the same design.
//!
//! Image layers store a local filesystem path, which obviously won't
//! resolve on the other person's machine, so every sync also carries the
//! raw bytes of every referenced image. The receiving side writes them into
//! its own temp-dir cache (named by a content hash, so repeat syncs of the
//! same image don't re-write it) and rewrites the incoming project's paths
//! to point at those local copies before handing it back to the app.

use crate::model::{Background, LayerKind, Project};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub const DEFAULT_PORT: u16 = 7878;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Host,
    Client,
}

pub enum NetEvent {
    PeerConnected,
    PeerDisconnected,
    ProjectReceived(Project),
    /// A collaborator's live pointer position, in project (canvas) space,
    /// plus their display name — sent continuously while their pointer is
    /// over the canvas, separate from the (much rarer) full project syncs.
    CursorReceived { x: f32, y: f32, name: String },
}

pub struct NetworkState {
    pub role: Role,
    pub label: String,
    pub events: Receiver<NetEvent>,
    outbound: Sender<OutboundMsg>,
    peers: Arc<Mutex<Vec<TcpStream>>>,
    /// Told to the host's accept loop to stop and release the listening
    /// socket; unused (but harmless to set) for a client, which has no
    /// listener of its own to stop.
    shutdown: Arc<AtomicBool>,
}

/// Actually tears the session down instead of just detaching the app's UI
/// from it: stops the host's accept loop (so the port is free again) and
/// shuts down every open connection (so any thread blocked in a read wakes
/// up immediately, and the other side sees a prompt disconnect rather than
/// a connection that silently goes nowhere).
impl Drop for NetworkState {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Ok(peers) = self.peers.lock() {
            for stream in peers.iter() {
                let _ = stream.shutdown(std::net::Shutdown::Both);
            }
        }
    }
}

enum OutboundMsg {
    Project(Project),
    Cursor { x: f32, y: f32, name: String },
}

impl NetworkState {
    pub fn send_project(&self, project: &Project) {
        let _ = self.outbound.send(OutboundMsg::Project(project.clone()));
    }

    pub fn send_cursor(&self, x: f32, y: f32, name: &str) {
        let _ = self.outbound.send(OutboundMsg::Cursor { x, y, name: name.to_string() });
    }
}

#[derive(Serialize, Deserialize)]
struct AssetBlob {
    path: String,
    bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
enum WireMessage {
    ProjectSync { project: Project, assets: Vec<AssetBlob> },
    Cursor { x: f32, y: f32, name: String },
}

/// This machine's LAN-facing IP address, best-effort. Connecting a UDP
/// socket doesn't actually send a packet — it just asks the OS which local
/// interface/address it would route through, which is exactly the address
/// the other person needs to reach this machine.
pub fn local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}

pub fn host(port: u16) -> std::io::Result<NetworkState> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    // Blocking accept() can't be interrupted from outside — the only way to
    // notice "please stop" is to poll non-blockingly instead.
    listener.set_nonblocking(true)?;
    let peers: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    let (event_tx, event_rx) = mpsc::channel();
    let (outbound_tx, outbound_rx) = mpsc::channel::<OutboundMsg>();
    let shutdown = Arc::new(AtomicBool::new(false));

    {
        let peers = Arc::clone(&peers);
        let event_tx = event_tx.clone();
        let shutdown = Arc::clone(&shutdown);
        thread::spawn(move || {
            while !shutdown.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = stream.set_nodelay(true);
                        // Accepted sockets can inherit the listener's own
                        // non-blocking mode on some platforms; the reader
                        // thread needs a normal blocking read.
                        let _ = stream.set_nonblocking(false);
                        if let Ok(clone) = stream.try_clone() {
                            peers.lock().unwrap().push(clone);
                            spawn_reader(stream, event_tx.clone());
                            let _ = event_tx.send(NetEvent::PeerConnected);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(_) => thread::sleep(std::time::Duration::from_millis(100)),
                }
            }
            // `listener` is dropped here, releasing the port.
        });
    }

    spawn_broadcaster(Arc::clone(&peers), outbound_rx);

    Ok(NetworkState {
        role: Role::Host,
        label: format!("Hosting on port {port}"),
        events: event_rx,
        outbound: outbound_tx,
        peers,
        shutdown,
    })
}

const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Starts connecting in the background and returns immediately — connecting
/// used to block the UI thread on `TcpStream::connect`, which has no
/// built-in timeout and can hang for a long time (a minute-plus on some
/// OSes) against an address nobody's listening on, making the app look
/// frozen with no feedback. Poll the returned receiver each frame instead.
pub fn join(addr: &str) -> Receiver<Result<NetworkState, String>> {
    let (result_tx, result_rx) = mpsc::channel();
    let addr = addr.to_string();
    thread::spawn(move || {
        let _ = result_tx.send(try_join(&addr));
    });
    result_rx
}

fn try_join(addr: &str) -> Result<NetworkState, String> {
    let socket_addr = addr
        .to_socket_addrs()
        .map_err(|e| format!("bad address: {e}"))?
        .next()
        .ok_or_else(|| "bad address: couldn't resolve it to anything".to_string())?;
    let stream = TcpStream::connect_timeout(&socket_addr, CONNECT_TIMEOUT).map_err(|e| e.to_string())?;
    let _ = stream.set_nodelay(true);
    let clone = stream.try_clone().map_err(|e| e.to_string())?;
    let peers = Arc::new(Mutex::new(vec![clone]));

    let (event_tx, event_rx) = mpsc::channel();
    spawn_reader(stream, event_tx.clone());
    let _ = event_tx.send(NetEvent::PeerConnected);

    let (outbound_tx, outbound_rx) = mpsc::channel::<OutboundMsg>();
    spawn_broadcaster(Arc::clone(&peers), outbound_rx);

    Ok(NetworkState {
        role: Role::Client,
        label: format!("Connected to {addr}"),
        events: event_rx,
        outbound: outbound_tx,
        peers,
        shutdown: Arc::new(AtomicBool::new(false)),
    })
}

fn spawn_reader(mut stream: TcpStream, event_tx: Sender<NetEvent>) {
    thread::spawn(move || loop {
        match read_message(&mut stream) {
            Ok(WireMessage::ProjectSync { mut project, assets }) => {
                apply_assets(&mut project, &assets);
                if event_tx.send(NetEvent::ProjectReceived(project)).is_err() {
                    break;
                }
            }
            Ok(WireMessage::Cursor { x, y, name }) => {
                if event_tx.send(NetEvent::CursorReceived { x, y, name }).is_err() {
                    break;
                }
            }
            Err(_) => {
                let _ = event_tx.send(NetEvent::PeerDisconnected);
                break;
            }
        }
    });
}

fn spawn_broadcaster(peers: Arc<Mutex<Vec<TcpStream>>>, outbound_rx: Receiver<OutboundMsg>) {
    thread::spawn(move || {
        while let Ok(outbound) = outbound_rx.recv() {
            let msg = match outbound {
                OutboundMsg::Project(project) => {
                    let assets = collect_assets(&project);
                    WireMessage::ProjectSync { project, assets }
                }
                OutboundMsg::Cursor { x, y, name } => WireMessage::Cursor { x, y, name },
            };
            let mut guard = peers.lock().unwrap();
            guard.retain_mut(|stream| write_message(stream, &msg).is_ok());
        }
    });
}

fn read_message(stream: &mut TcpStream) -> std::io::Result<WireMessage> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    serde_json::from_slice(&buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn write_message(stream: &mut TcpStream, msg: &WireMessage) -> std::io::Result<()> {
    let json = serde_json::to_vec(msg).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    stream.write_all(&(json.len() as u32).to_be_bytes())?;
    stream.write_all(&json)?;
    Ok(())
}

fn collect_assets(project: &Project) -> Vec<AssetBlob> {
    let mut seen = HashSet::new();
    let mut blobs = Vec::new();
    let mut add = |path: &str| {
        if path.is_empty() || !seen.insert(path.to_string()) {
            return;
        }
        if let Ok(bytes) = std::fs::read(path) {
            blobs.push(AssetBlob { path: path.to_string(), bytes });
        }
    };
    if let Background::Image { path } = &project.background {
        add(path);
    }
    for layer in &project.layers {
        match &layer.kind {
            LayerKind::Image(img) => add(&img.path),
            LayerKind::DeviceFrame(frame) => {
                if let Some(p) = &frame.custom_image_path {
                    add(p);
                }
            }
            _ => {}
        }
    }
    blobs
}

fn cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("mockup_studio_synced_assets");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Writes every asset blob to a content-addressed local file (skipping ones
/// already cached) and rewrites `project`'s image paths from the sender's
/// (foreign) paths to these local ones.
fn apply_assets(project: &mut Project, assets: &[AssetBlob]) {
    if assets.is_empty() {
        return;
    }
    let dir = cache_dir();
    let mut remap: HashMap<String, String> = HashMap::new();
    for asset in assets {
        let mut hasher = DefaultHasher::new();
        asset.bytes.hash(&mut hasher);
        let hash = hasher.finish();
        let ext = std::path::Path::new(&asset.path).extension().and_then(|e| e.to_str()).unwrap_or("bin");
        let local_path = dir.join(format!("{hash:016x}.{ext}"));
        if !local_path.exists() {
            let _ = std::fs::write(&local_path, &asset.bytes);
        }
        remap.insert(asset.path.clone(), local_path.to_string_lossy().to_string());
    }
    if let Background::Image { path } = &mut project.background {
        if let Some(new_path) = remap.get(path) {
            *path = new_path.clone();
        }
    }
    for layer in &mut project.layers {
        match &mut layer.kind {
            LayerKind::Image(img) => {
                if let Some(new_path) = remap.get(&img.path) {
                    img.path = new_path.clone();
                }
            }
            LayerKind::DeviceFrame(frame) => {
                if let Some(p) = &mut frame.custom_image_path {
                    if let Some(new_path) = remap.get(p) {
                        *p = new_path.clone();
                    }
                }
            }
            _ => {}
        }
    }
}
