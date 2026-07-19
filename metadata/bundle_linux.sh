#!/bin/bash
set -e

VERSION=${1:-"0.1.0"}
echo "Creating Linux Debian Package for version $VERSION..."

PKG_DIR="church-presenter_${VERSION}_amd64"
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/lib/x86_64-linux-gnu"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/512x512/apps"

# Copy binary
cp target/release/church-presenter "$PKG_DIR/usr/bin/"

# Copy libndi.so.4
cp target/release/deps/libndi.so.4 "$PKG_DIR/usr/lib/x86_64-linux-gnu/"

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
rm -rf "$PKG_DIR"

echo "Linux Debian packaging complete."

# Build RPM Package
if command -v rpmbuild >/dev/null 2>&1; then
    echo "Creating Linux RPM Package for version $VERSION..."
    
    # Create temporary sources dir
    rm -rf rpm-sources rpm-build
    mkdir -p rpm-sources
    
    # Copy source files
    cp target/release/church-presenter rpm-sources/
    cp target/release/deps/libndi.so.4 rpm-sources/
    cp metadata/play.png rpm-sources/church-presenter.png
    
    # Create desktop entry in rpm-sources
    cat <<EOF > "rpm-sources/church-presenter.desktop"
[Desktop Entry]
Type=Application
Name=Church Presenter
Comment=Lightweight, high-performance church presentation software
Exec=church-presenter
Icon=church-presenter
Terminal=false
Categories=Utility;Presentation;
EOF
    
    # Build RPM
    rpmbuild --define "version $VERSION" \
             --define "_sourcedir $(pwd)/rpm-sources" \
             --define "_rpmdir $(pwd)" \
             --define "_topdir $(pwd)/rpm-build" \
             -bb metadata/church-presenter.spec
             
    # Move and rename RPM package
    mv x86_64/church-presenter-${VERSION}-1.*.rpm church-presenter-setup.rpm
    rm -rf x86_64 rpm-sources rpm-build
    echo "Linux RPM packaging complete."
else
    echo "rpmbuild not found. Skipping RPM package generation."
fi
