#!/bin/bash
set -e

echo "Creating Windows Bundle..."
BUNDLE_DIR="dist-windows-bundle"
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"

# Copy executable
cp target/release/church-presenter.exe "$BUNDLE_DIR/"

# Copy dependencies
# We use ldd to find all dlls in /mingw64/bin and copy them
echo "Copying DLL dependencies..."
ldd target/release/church-presenter.exe | grep -i "/mingw64/bin" | awk '{print $3}' | while read -r dll; do
    # Convert windows path from ldd if needed, though in msys2 it is usually /mingw64/...
    if [ -f "$dll" ]; then
        cp "$dll" "$BUNDLE_DIR/"
    fi
done

# Copy glib schemas
echo "Copying glib schemas..."
mkdir -p "$BUNDLE_DIR/share/glib-2.0/schemas"
cp /mingw64/share/glib-2.0/schemas/gschemas.compiled "$BUNDLE_DIR/share/glib-2.0/schemas/"

# Copy gdk-pixbuf loaders
echo "Copying gdk-pixbuf loaders..."
mkdir -p "$BUNDLE_DIR/lib/gdk-pixbuf-2.0/2.10.0"
cp -r /mingw64/lib/gdk-pixbuf-2.0/2.10.0/* "$BUNDLE_DIR/lib/gdk-pixbuf-2.0/2.10.0/"

# Copy GTK assets (icons, themes, etc.)
echo "Copying GTK shared assets..."
mkdir -p "$BUNDLE_DIR/share/icons"
cp -r /mingw64/share/icons/Adwaita "$BUNDLE_DIR/share/icons/" || true
cp -r /mingw64/share/icons/hicolor "$BUNDLE_DIR/share/icons/" || true

echo "Bundle created successfully."
