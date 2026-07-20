#!/bin/bash
set -e

VERSION=${1:-"0.1.0"}
echo "Creating macOS App Bundle for version $VERSION..."

APP_DIR="Church Presenter.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copy binary (note that the renamed binary is now church-presenter)
cp target/release/church-presenter "$APP_DIR/Contents/MacOS/"

# Copy KJV.sqlite database into Resources
cp KJV.sqlite "$APP_DIR/Contents/Resources/"

# Generate ICNS from PNG
echo "Generating ICNS icon..."
mkdir -p play.iconset
sips -z 16 16     metadata/play.png --out play.iconset/icon_16x16.png
sips -z 32 32     metadata/play.png --out play.iconset/icon_16x16@2x.png
sips -z 32 32     metadata/play.png --out play.iconset/icon_32x32.png
sips -z 64 64     metadata/play.png --out play.iconset/icon_32x32@2x.png
sips -z 128 128   metadata/play.png --out play.iconset/icon_128x128.png
sips -z 256 256   metadata/play.png --out play.iconset/icon_128x128@2x.png
sips -z 256 256   metadata/play.png --out play.iconset/icon_256x256.png
sips -z 512 512   metadata/play.png --out play.iconset/icon_256x256@2x.png
sips -z 512 512   metadata/play.png --out play.iconset/icon_512x512.png
sips -z 1024 1024 metadata/play.png --out play.iconset/icon_512x512@2x.png
iconutil -c icns play.iconset
mv play.icns "$APP_DIR/Contents/Resources/play.icns"
rm -rf play.iconset

# Create Info.plist with app metadata
cat <<EOF > "$APP_DIR/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleDisplayName</key>
    <string>Church Presenter</string>
    <key>CFBundleExecutable</key>
    <string>church-presenter</string>
    <key>CFBundleIconFile</key>
    <string>play.icns</string>
    <key>CFBundleIdentifier</key>
    <string>org.thruqe.churchpresenter</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Church Presenter</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Code signing (optional, runs if certificate environment variables are present)
if [ -n "$MACOS_CERTIFICATE" ] && [ -n "$MACOS_CERTIFICATE_PWD" ]; then
    echo "Importing macOS signing certificate..."
    KEYCHAIN="build.keychain"
    security create-keychain -p "" "$KEYCHAIN"
    security default-keychain -s "$KEYCHAIN"
    security unlock-keychain -p "" "$KEYCHAIN"
    
    echo "$MACOS_CERTIFICATE" | base64 --decode > certificate.p12
    security import certificate.p12 -k "$KEYCHAIN" -P "$MACOS_CERTIFICATE_PWD" -T /usr/bin/codesign
    security set-key-partition-list -S apple-tool:,apple: -s -k "" "$KEYCHAIN"
    
    echo "Signing app bundle..."
    codesign --force --options runtime --sign "Developer ID Application: Thruqe" "$APP_DIR"
fi

# Package into a DMG
echo "Packaging into DMG..."
rm -f "church-presenter-setup.dmg"
hdiutil create -volname "Church Presenter" -srcfolder "$APP_DIR" -ov -format UDZO "church-presenter-setup.dmg"

# Notarization (optional, runs if Apple ID credentials are provided)
if [ -n "$MACOS_NOTARIZATION_APPLE_ID" ] && [ -n "$MACOS_NOTARIZATION_PASSWORD" ]; then
    echo "Submitting DMG for notarization..."
    xcrun notarytool submit church-presenter-setup.dmg --apple-id "$MACOS_NOTARIZATION_APPLE_ID" --password "$MACOS_NOTARIZATION_PASSWORD" --team-id "$MACOS_TEAM_ID" --wait
    echo "Stapling notarization ticket..."
    xcrun stapler staple church-presenter-setup.dmg
fi

echo "macOS packaging complete."
