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
mkdir -p "$PKG_DIR/usr/share/pixmaps"

# Icon sizes required for full hicolor coverage (taskbar, launcher, Alt+Tab, file manager…)
ICON_SIZES=(16 24 32 48 64 128 256 512)
for SIZE in "${ICON_SIZES[@]}"; do
    mkdir -p "$PKG_DIR/usr/share/icons/hicolor/${SIZE}x${SIZE}/apps"
done

# Copy binary
cp target/release/church-presenter "$PKG_DIR/usr/bin/"

# Copy libndi.so.4
cp target/release/deps/libndi.so.4 "$PKG_DIR/usr/lib/x86_64-linux-gnu/"

# Copy KJV.sqlite
mkdir -p "$PKG_DIR/usr/share/church-presenter"
cp KJV.sqlite "$PKG_DIR/usr/share/church-presenter/KJV.sqlite"

# Generate all required icon sizes from the source PNG using ImageMagick
echo "Generating application icons at all required sizes..."
for SIZE in "${ICON_SIZES[@]}"; do
    DEST="$PKG_DIR/usr/share/icons/hicolor/${SIZE}x${SIZE}/apps/church-presenter.png"
    if command -v rsvg-convert >/dev/null 2>&1 && [ -f metadata/play.svg ]; then
        # Preferred: render direct from SVG for pixel-perfect quality at every size
        rsvg-convert -w "$SIZE" -h "$SIZE" metadata/play.svg -o "$DEST"
    else
        # Fallback: resize from the pre-rendered 512px PNG
        convert metadata/play.png -resize "${SIZE}x${SIZE}" "$DEST"
    fi
done

# Also copy a 48x48 version to /usr/share/pixmaps for legacy compatibility
# (some desktop environments / app launchers still look there)
cp "$PKG_DIR/usr/share/icons/hicolor/48x48/apps/church-presenter.png" \
   "$PKG_DIR/usr/share/pixmaps/church-presenter.png"

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
Depends: libgtk-4-1, imagemagick
Description: Church Presenter is a lightweight, high-performance presentation software built with Rust and GTK4.
 It allows church media teams to present Scripture verses and song lyrics on local displays as well as broadcast them as NDI streams.
EOF

# postinst: refresh icon & desktop caches after install so the icon appears immediately
cat <<'POSTINST' > "$PKG_DIR/DEBIAN/postinst"
#!/bin/sh
set -e

# Refresh the XDG icon cache so every icon size becomes visible
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor
fi

# Update the application menu database
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications
fi

exit 0
POSTINST
chmod 0755 "$PKG_DIR/DEBIAN/postinst"

# postrm: clean up caches after removal
cat <<'POSTRM' > "$PKG_DIR/DEBIAN/postrm"
#!/bin/sh
set -e

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi

exit 0
POSTRM
chmod 0755 "$PKG_DIR/DEBIAN/postrm"

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
    mkdir -p rpm-sources/icons

    # Copy source files
    cp target/release/church-presenter rpm-sources/
    cp target/release/deps/libndi.so.4 rpm-sources/

    # Generate all icon sizes for RPM
    echo "Generating icons for RPM..."
    for SIZE in "${ICON_SIZES[@]}"; do
        convert metadata/play.png \
            -resize "${SIZE}x${SIZE}" \
            "rpm-sources/icons/church-presenter_${SIZE}.png"
    done

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
