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

# Bundle NDI SDK DLL
# The NDI SDK installer extracts to a known location; copy the runtime DLL
echo "Copying NDI SDK DLL..."
NDI_DLL_PATHS=(
    "/c/Program Files/NDI/NDI 6 Runtime/v6/Processing.NDI.Lib.x64.dll"
    "/c/Program Files/NDI/NDI 5 Runtime/v5/Processing.NDI.Lib.x64.dll"
    "C:/Program Files/NDI/NDI 6 Runtime/v6/Processing.NDI.Lib.x64.dll"
    "C:/Program Files/NDI/NDI 5 Runtime/v5/Processing.NDI.Lib.x64.dll"
)
NDI_FOUND=false
for NDI_DLL in "${NDI_DLL_PATHS[@]}"; do
    if [ -f "$NDI_DLL" ]; then
        cp "$NDI_DLL" "$BUNDLE_DIR/"
        echo "  Copied: $NDI_DLL"
        NDI_FOUND=true
        break
    fi
done
if [ "$NDI_FOUND" = false ]; then
    echo "WARNING: NDI DLL not found in standard paths. Searching system..."
    # Try a broader search under /c/Program Files
    FOUND_DLL=$(find "/c/Program Files/NDI" -name "Processing.NDI.Lib.x64.dll" 2>/dev/null | head -1)
    if [ -n "$FOUND_DLL" ]; then
        cp "$FOUND_DLL" "$BUNDLE_DIR/"
        echo "  Copied (found): $FOUND_DLL"
    else
        echo "ERROR: Processing.NDI.Lib.x64.dll not found. Bundle may be incomplete."
        exit 1
    fi
fi

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

# Bundle the application icon into hicolor so Windows Explorer and
# the taskbar can display it at every required size.
echo "Copying application icon..."
mkdir -p "$BUNDLE_DIR/share/icons/hicolor/scalable/apps"
cp metadata/play.ico "$BUNDLE_DIR/share/icons/hicolor/scalable/apps/church-presenter.ico"
# Also drop the .ico at the root so the installer can reference it directly
cp metadata/play.ico "$BUNDLE_DIR/church-presenter.ico"

echo "Bundle created successfully."
