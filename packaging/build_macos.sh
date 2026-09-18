#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

APP_NAME="Mockup Studio"
BIN_NAME="mockup_studio"
BUNDLE_ID="com.mockupstudio.app"
VERSION="0.6.0"

echo "==> Building release binary (Apple Silicon)"
rustup target add aarch64-apple-darwin >/dev/null 2>&1 || true
cargo build --release --target aarch64-apple-darwin

echo "==> Building release binary (Intel)"
rustup target add x86_64-apple-darwin >/dev/null 2>&1 || true
cargo build --release --target x86_64-apple-darwin

echo "==> Combining into a universal binary"
UNIVERSAL_BIN="target/universal/${BIN_NAME}"
mkdir -p "target/universal"
lipo -create -output "$UNIVERSAL_BIN" \
    "target/aarch64-apple-darwin/release/${BIN_NAME}" \
    "target/x86_64-apple-darwin/release/${BIN_NAME}"
lipo -info "$UNIVERSAL_BIN"

APP_DIR="packaging/dist/${APP_NAME}.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cp "$UNIVERSAL_BIN" "$APP_DIR/Contents/MacOS/${BIN_NAME}"
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

# A "Developer ID Application" cert (not "Apple Distribution" — that's for
# the App Store) is what notarization requires. Falls back to ad-hoc signing
# if one isn't installed, so this script still works for local test builds.
DEVELOPER_ID=$(security find-identity -v -p codesigning 2>/dev/null | grep '"Developer ID Application:' | head -1 | sed -E 's/.*"(.*)"/\1/' || true)
NOTARY_PROFILE="mockup-studio-notary"

if [ -n "$DEVELOPER_ID" ]; then
    echo "==> Signing with Developer ID: $DEVELOPER_ID"
    codesign --force --deep --options runtime --timestamp --sign "$DEVELOPER_ID" "$APP_DIR"
else
    echo "==> No Developer ID Application cert found — signing ad-hoc (won't pass notarization)"
    codesign --force --deep --sign - "$APP_DIR"
fi

echo "==> Building DMG"
DMG_STAGING="packaging/dist/dmg_staging"
rm -rf "$DMG_STAGING"
mkdir -p "$DMG_STAGING"
cp -R "$APP_DIR" "$DMG_STAGING/"
ln -s /Applications "$DMG_STAGING/Applications"

DMG_PATH="packaging/dist/MockupStudio-macOS.dmg"
rm -f "$DMG_PATH"
hdiutil create -volname "${APP_NAME}" -srcfolder "$DMG_STAGING" -ov -format UDZO "$DMG_PATH"

if [ -n "$DEVELOPER_ID" ] && xcrun notarytool history --keychain-profile "$NOTARY_PROFILE" >/dev/null 2>&1; then
    echo "==> Submitting for notarization (this can take a few minutes)"
    xcrun notarytool submit "$DMG_PATH" --keychain-profile "$NOTARY_PROFILE" --wait
    echo "==> Stapling notarization ticket"
    xcrun stapler staple "$DMG_PATH"
else
    echo "==> Skipping notarization (no Developer ID cert and/or no '$NOTARY_PROFILE' keychain profile)"
fi

echo "==> Done"
echo "App:  $APP_DIR"
echo "DMG:  $DMG_PATH"
