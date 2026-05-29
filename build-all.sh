#!/bin/bash
# Build all platform targets locally (macOS native, cross for linux/win via cross)
set -e

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
)

DIST_DIR="dist"
mkdir -p "$DIST_DIR"

for target in "${TARGETS[@]}"; do
    echo "=== Building $target ==="

    # macOS targets can be built natively
    if [[ "$target" == *apple* ]]; then
        rustup target add "$target" 2>/dev/null || true
        cargo build --release --target "$target"
    else
        # Linux/Windows require cross (Docker-based)
        if ! command -v cross &>/dev/null; then
            echo "Installing cross..."
            cargo install cross --git https://github.com/cross-rs/cross
        fi
        cross build --release --target "$target"
    fi

    # Copy artifacts to dist/
    case "$target" in
        *linux*)
            cp "target/$target/release/bright-sdk-downloader" "$DIST_DIR/bright-sdk-downloader-linux-x64"
            cp "target/$target/release/libbright_sdk_download.so" "$DIST_DIR/"
            ;;
        *apple*x86*)
            cp "target/$target/release/bright-sdk-downloader" "$DIST_DIR/bright-sdk-downloader-macos-x64"
            cp "target/$target/release/libbright_sdk_download.dylib" "$DIST_DIR/libbright_sdk_download-x64.dylib"
            ;;
        *apple*aarch64*)
            cp "target/$target/release/bright-sdk-downloader" "$DIST_DIR/bright-sdk-downloader-macos-arm64"
            cp "target/$target/release/libbright_sdk_download.dylib" "$DIST_DIR/libbright_sdk_download-arm64.dylib"
            ;;
        *windows*)
            cp "target/$target/release/bright-sdk-downloader.exe" "$DIST_DIR/bright-sdk-downloader-win-x64.exe"
            cp "target/$target/release/bright_sdk_download.dll" "$DIST_DIR/"
            ;;
    esac
done

echo ""
echo "=== Build complete ==="
ls -lh "$DIST_DIR/"
