#!/bin/bash
set -euo pipefail

# Build a signed macOS .app bundle for Alocir.
# Usage: ./scripts/build-macos.sh [--sign IDENTITY]
#   --sign IDENTITY   Code-sign with the given identity (e.g. "Developer ID Application: ...")
#                     Defaults to ad-hoc signing (-) if omitted.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_DIR/target/release"
BUNDLE_DIR="$BUILD_DIR/Alocir.app"
SIGN_IDENTITY="-"

while [[ $# -gt 0 ]]; do
    case $1 in
        --sign) SIGN_IDENTITY="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo "==> Building release binary..."
cargo build --release

echo "==> Creating app bundle..."
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

# Copy binary
cp "$BUILD_DIR/alocir" "$BUNDLE_DIR/Contents/MacOS/alocir"

# Copy Info.plist
cp "$PROJECT_DIR/macos/Info.plist" "$BUNDLE_DIR/Contents/Info.plist"

# Generate .icns from SVG
echo "==> Generating app icon..."
ICONSET_DIR=$(mktemp -d)/AppIcon.iconset
mkdir -p "$ICONSET_DIR"

# Use sips to convert SVG -> PNG at required sizes.
# sips can't read SVG directly, so we use the built-in qlmanage or a temp PNG.
# Try qlmanage first (available on macOS), fall back to a simple approach.
SVG="$PROJECT_DIR/assets/logo.svg"
TEMP_PNG=$(mktemp /tmp/alocir_icon_XXXXXX.png)

if command -v rsvg-convert &>/dev/null; then
    rsvg-convert -w 1024 -h 1024 "$SVG" -o "$TEMP_PNG"
elif command -v qlmanage &>/dev/null; then
    # qlmanage generates a preview; extract the PNG.
    TEMP_QL_DIR=$(mktemp -d)
    qlmanage -t -s 1024 -o "$TEMP_QL_DIR" "$SVG" &>/dev/null || true
    QL_PNG=$(find "$TEMP_QL_DIR" -name "*.png" | head -1)
    if [[ -n "$QL_PNG" && -f "$QL_PNG" ]]; then
        cp "$QL_PNG" "$TEMP_PNG"
    else
        echo "Warning: Could not convert SVG to PNG. Skipping .icns generation."
        echo "         Install librsvg (brew install librsvg) for best results."
        TEMP_PNG=""
    fi
    rm -rf "$TEMP_QL_DIR"
else
    echo "Warning: No SVG converter found. Skipping .icns generation."
    echo "         Install librsvg (brew install librsvg) for best results."
    TEMP_PNG=""
fi

if [[ -n "$TEMP_PNG" && -f "$TEMP_PNG" ]]; then
    for SIZE in 16 32 64 128 256 512 1024; do
        sips -z $SIZE $SIZE "$TEMP_PNG" --out "$ICONSET_DIR/icon_${SIZE}x${SIZE}.png" &>/dev/null
    done
    # macOS expects @2x variants
    cp "$ICONSET_DIR/icon_32x32.png"   "$ICONSET_DIR/icon_16x16@2x.png"
    cp "$ICONSET_DIR/icon_64x64.png"   "$ICONSET_DIR/icon_32x32@2x.png"
    cp "$ICONSET_DIR/icon_256x256.png" "$ICONSET_DIR/icon_128x128@2x.png"
    cp "$ICONSET_DIR/icon_512x512.png" "$ICONSET_DIR/icon_256x256@2x.png"
    cp "$ICONSET_DIR/icon_1024x1024.png" "$ICONSET_DIR/icon_512x512@2x.png"
    # Remove non-standard sizes
    rm -f "$ICONSET_DIR/icon_64x64.png" "$ICONSET_DIR/icon_1024x1024.png"

    iconutil -c icns "$ICONSET_DIR" -o "$BUNDLE_DIR/Contents/Resources/AppIcon.icns"
    echo "    Icon generated."
    rm -f "$TEMP_PNG"
fi

# Sign the bundle
echo "==> Signing with identity: $SIGN_IDENTITY"
codesign --force --deep -s "$SIGN_IDENTITY" "$BUNDLE_DIR"

echo ""
echo "==> Done! App bundle created at:"
echo "    $BUNDLE_DIR"
echo ""
echo "    To install: cp -r \"$BUNDLE_DIR\" /Applications/"
