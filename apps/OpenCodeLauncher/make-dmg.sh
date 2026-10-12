#!/bin/bash
set -euo pipefail

APP_NAME="OpenCodeLauncher"
BUNDLE_ID="com.opencode.launcher"
VERSION="1.0.0"
DISPLAY_NAME="OpenCode Launcher"

ROOT="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$ROOT/build"
APP="$BUILD_DIR/$APP_NAME.app"
DMG_STAGE="$BUILD_DIR/dmg"
DMG_OUT="$ROOT/$APP_NAME-$VERSION.dmg"

echo "==> Build release"
swift build -c release --package-path "$ROOT"

BIN="$(swift build -c release --package-path "$ROOT" --show-bin-path)/$APP_NAME"
if [ ! -f "$BIN" ]; then
  echo "Binaire introuvable: $BIN" >&2
  exit 1
fi

echo "==> Assemblage du bundle .app"
rm -rf "$APP" "$DMG_STAGE" "$DMG_OUT"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/$APP_NAME"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>$DISPLAY_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
PLIST

echo "==> Signature ad-hoc"
codesign --force --deep --sign - "$APP"

echo "==> Préparation du DMG"
mkdir -p "$DMG_STAGE"
cp -R "$APP" "$DMG_STAGE/"
ln -s /Applications "$DMG_STAGE/Applications"

hdiutil create \
  -volname "$DISPLAY_NAME" \
  -srcfolder "$DMG_STAGE" \
  -ov -format UDZO \
  "$DMG_OUT"

rm -rf "$DMG_STAGE"

echo ""
echo "DMG prêt : $DMG_OUT"
