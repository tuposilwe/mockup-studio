#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

APP_NAME="Mockup Studio"
BIN_NAME="mockup_studio"
BUNDLE_ID="com.mockupstudio.app"
VERSION="0.2.0"

echo "==> Building release binary"
cargo build --release

APP_DIR="packaging/dist/${APP_NAME}.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cp "target/release/${BIN_NAME}" "$APP_DIR/Contents/MacOS/${BIN_NAME}"
cp "packaging/AppIcon.icns" "$APP_DIR/Contents/Resources/AppIcon.icns"

cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>${BIN_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon.icns</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

chmod +x "$APP_DIR/Contents/MacOS/${BIN_NAME}"

echo "==> Signing (ad-hoc)"
codesign --force --deep --sign - "$APP_DIR"

echo "==> Building DMG"
DMG_STAGING="packaging/dist/dmg_staging"
rm -rf "$DMG_STAGING"
mkdir -p "$DMG_STAGING"
cp -R "$APP_DIR" "$DMG_STAGING/"
ln -s /Applications "$DMG_STAGING/Applications"

DMG_PATH="packaging/dist/MockupStudio-macOS.dmg"
rm -f "$DMG_PATH"
hdiutil create -volname "${APP_NAME}" -srcfolder "$DMG_STAGING" -ov -format UDZO "$DMG_PATH"

echo "==> Done"
echo "App:  $APP_DIR"
echo "DMG:  $DMG_PATH"
