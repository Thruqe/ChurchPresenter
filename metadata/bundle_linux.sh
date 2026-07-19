#!/bin/bash
set -e

VERSION=${1:-"0.1.0"}
echo "Creating Linux Debian Package for version $VERSION..."

PKG_DIR="church-presenter_${VERSION}_amd64"
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/512x512/apps"

# Copy binary
cp target/release/church-presenter "$PKG_DIR/usr/bin/"

# Copy icon
cp metadata/play.png "$PKG_DIR/usr/share/icons/hicolor/512x512/apps/church-presenter.png"

# Create .desktop entry
cat <<EOF > "$PKG_DIR/usr/share/applications/church-presenter.desktop"
[Desktop Entry]
Type=Application
Name=Church Presenter
Comment=Lightweight, high-performance church presentation software
Exec=church-presenter
Icon=church-presenter
Terminal=false
Categories=Utility;Presentation;
EOF

# Create Debian control file with metadata
cat <<EOF > "$PKG_DIR/DEBIAN/control"
Package: church-presenter
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Thruqe <danielpeter0039@gmail.com>
Depends: libgtk-4-1
Description: Church Presenter is a lightweight, high-performance presentation software built with Rust and GTK4.
 It allows church media teams to present Scripture verses and song lyrics on local displays as well as broadcast them as NDI streams.
EOF

# Build package
dpkg-deb --build "$PKG_DIR"
mv "${PKG_DIR}.deb" "church-presenter-setup.deb"

echo "Linux packaging complete."
