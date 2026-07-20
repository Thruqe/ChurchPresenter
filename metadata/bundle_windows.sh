#!/bin/bash
set -e

echo "Creating Windows Bundle..."
BUNDLE_DIR="dist-windows-bundle"
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"

# Copy executable
cp target/release/church-presenter.exe "$BUNDLE_DIR/"

# ── Recursive DLL collector ─────────────────────────────────────────────────
#
# Root cause of "missing DLL" errors: ldd only shows *direct* dependencies.
# GTK4/GLib/gdk-pixbuf have deep transitive dependency chains (e.g. libtiff
# → zlib1, libffi; libfontconfig → libexpat; etc.) that never appear at the
# top level.  Playing whack-a-mole by hard-coding DLL names one-by-one is
# not maintainable.
#
# Solution: BFS over the full dependency graph.
#   1. Start with the main exe in a worklist.
#   2. Run ldd on the current binary, collect every /mingw64/bin DLL.
#   3. For each unseen DLL: copy it, add it to the worklist.
#   4. Repeat until the worklist is empty.
#
# This guarantees the full transitive closure in one shot, for any depth.
# ───────────────────────────────────────────────────────────────────────────

MINGW_BIN="/mingw64/bin"

echo "Collecting full transitive DLL dependency tree (BFS)..."

# Associative array used as a seen-set (bash 4+, available in MSYS2)
declare -A SEEN_DLLS

# BFS worklist (array used as a queue)
WORKLIST=("target/release/church-presenter.exe")
DLL_COUNT=0

while [ "${#WORKLIST[@]}" -gt 0 ]; do
    # Pop the first element
    BINARY="${WORKLIST[0]}"
    WORKLIST=("${WORKLIST[@]:1}")

    # Find all direct MinGW64 DLL dependencies of this binary
    while IFS= read -r DLL; do
        [ -f "$DLL" ] || continue
        DLL_KEY="$(basename "$DLL")"
        # Skip if already seen
        [ -n "${SEEN_DLLS[$DLL_KEY]+x}" ] && continue

        # Mark seen, copy, enqueue
        SEEN_DLLS["$DLL_KEY"]=1
        cp "$DLL" "$BUNDLE_DIR/"
        WORKLIST+=("$DLL")   # recurse into this DLL's own deps
        DLL_COUNT=$((DLL_COUNT + 1))
        echo "  + $DLL_KEY"
    done < <(ldd "$BINARY" 2>/dev/null | grep -i "${MINGW_BIN}" | awk '{print $3}')
done

echo "Collected ${DLL_COUNT} DLLs (full transitive closure)."

# ── gdk-pixbuf loaders (plugins, also have their own DLL deps) ───────────────
echo "Copying gdk-pixbuf loaders..."
mkdir -p "$BUNDLE_DIR/lib/gdk-pixbuf-2.0/2.10.0"
cp -r /mingw64/lib/gdk-pixbuf-2.0/2.10.0/* "$BUNDLE_DIR/lib/gdk-pixbuf-2.0/2.10.0/"

echo "Collecting DLLs required by gdk-pixbuf loaders..."
while IFS= read -r LOADER; do
    while IFS= read -r DLL; do
        [ -f "$DLL" ] || continue
        DLL_KEY="$(basename "$DLL")"
        [ -n "${SEEN_DLLS[$DLL_KEY]+x}" ] && continue
        SEEN_DLLS["$DLL_KEY"]=1
        cp "$DLL" "$BUNDLE_DIR/"
        DLL_COUNT=$((DLL_COUNT + 1))
        echo "  + $DLL_KEY (loader dep)"
    done < <(ldd "$LOADER" 2>/dev/null | grep -i "${MINGW_BIN}" | awk '{print $3}')
done < <(find "$BUNDLE_DIR/lib/gdk-pixbuf-2.0" -name "*.dll" 2>/dev/null)

echo "Total DLLs after loader pass: ${DLL_COUNT}"

# ── NDI Runtime DLL ─────────────────────────────────────────────────────────
# Not a MinGW package — installed separately by the NDI Runtime installer.
echo "Copying NDI Runtime DLL..."
NDI_FOUND=false
for NDI_DLL in \
    "/c/Program Files/NDI/NDI 6 Runtime/v6/Processing.NDI.Lib.x64.dll" \
    "/c/Program Files/NDI/NDI 5 Runtime/v5/Processing.NDI.Lib.x64.dll" \
    "C:/Program Files/NDI/NDI 6 Runtime/v6/Processing.NDI.Lib.x64.dll" \
    "C:/Program Files/NDI/NDI 5 Runtime/v5/Processing.NDI.Lib.x64.dll"
do
    if [ -f "$NDI_DLL" ]; then
        cp "$NDI_DLL" "$BUNDLE_DIR/"
        echo "  Copied: $(basename "$NDI_DLL")"
        NDI_FOUND=true
        break
    fi
done
if [ "$NDI_FOUND" = false ]; then
    FOUND_DLL=$(find "/c/Program Files/NDI" -name "Processing.NDI.Lib.x64.dll" 2>/dev/null | head -1)
    if [ -n "$FOUND_DLL" ]; then
        cp "$FOUND_DLL" "$BUNDLE_DIR/"
        echo "  Copied (search): $(basename "$FOUND_DLL")"
    else
        echo "ERROR: Processing.NDI.Lib.x64.dll not found. Bundle may be incomplete."
        exit 1
    fi
fi

# ── GTK data assets ──────────────────────────────────────────────────────────
echo "Copying GLib schemas..."
mkdir -p "$BUNDLE_DIR/share/glib-2.0/schemas"
cp /mingw64/share/glib-2.0/schemas/gschemas.compiled "$BUNDLE_DIR/share/glib-2.0/schemas/"

echo "Copying GTK icon themes..."
mkdir -p "$BUNDLE_DIR/share/icons"
cp -r /mingw64/share/icons/Adwaita "$BUNDLE_DIR/share/icons/" || true
cp -r /mingw64/share/icons/hicolor "$BUNDLE_DIR/share/icons/" || true

# ── SQLite database ──────────────────────────────────────────────────────────
echo "Copying KJV.sqlite database..."
if [ -f "KJV.sqlite" ]; then
    cp KJV.sqlite "$BUNDLE_DIR/KJV.sqlite"
    mkdir -p "$BUNDLE_DIR/saves"
    cp KJV.sqlite "$BUNDLE_DIR/saves/KJV.sqlite"
    echo "  Copied KJV.sqlite"
fi

# ── Application icon ─────────────────────────────────────────────────────────
echo "Copying application icon..."
cp metadata/play.ico "$BUNDLE_DIR/church-presenter.ico"

echo ""
echo "Bundle created successfully."
echo "  Total DLLs : ${DLL_COUNT}"
echo "  Location   : ${BUNDLE_DIR}/"
