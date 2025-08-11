#!/bin/bash
set -euo pipefail

# === Configuration ===
RUST_VERSION="1.85.0"
SRC_DIR="/tmp/source/rust"
QNX_DIR="$HOME/qnx710"
TOOLCHAIN_NAME="qnx7.1_rust"
INSTALL_DIR="$HOME/rust_toolchains/"

# === Detect host arch (x86_64 or aarch64) ===
HOST_ARCH="$(uname -m)"
case "$HOST_ARCH" in
    x86_64) HOST_ARCH="x86_64" ;;
    aarch64) HOST_ARCH="aarch64" ;;
    *) echo "Unsupported host arch: $HOST_ARCH"; exit 1 ;;
esac

# === Setup QNX SDP ===
if [[ ! -d "$QNX_DIR" ]]; then
    echo "Error: QNX SDP expected at $QNX_DIR"
    exit 1
fi

# === Clone Rust ===
if [[ ! -d "$SRC_DIR" ]]; then
    echo "[*] Cloning Rust $RUST_VERSION..."
    git clone https://github.com/rust-lang/rust.git -b "$RUST_VERSION" --depth 1 "$SRC_DIR"
fi

cd "$SRC_DIR"

# Set QNX environment variables
source ${QNX_DIR}/qnxsdp-env.sh

# === Write build config ===
if [[ ! -f config.toml ]]; then
    echo "[*] Writing config.toml..."
    cat > config.toml <<EOF
change-id = 999999
[build]
extended = true
[llvm]
download-ci-llvm = false
EOF
fi

# === Build environment for QNX targets ===
export CC_x86_64_pc_nto_qnx710=qcc
export CFLAGS_x86_64_pc_nto_qnx710=-Vgcc_ntox86_64_cxx
export CXX_x86_64_pc_nto_qnx710=qcc
export AR_x86_64_pc_nto_qnx710=ntox86_64-ar

export CC_aarch64_unknown_nto_qnx710=qcc
export CFLAGS_aarch64_unknown_nto_qnx710=-Vgcc_ntoaarch64le_cxx
export CXX_aarch64_unknown_nto_qnx710=qcc
export AR_aarch64_unknown_nto_qnx710=ntoaarch64-ar

NATIVE_TARGET="${HOST_ARCH}-unknown-linux-gnu"

# === Build Rust ===
echo "[*] Building Rust for QNX targets..."
./x.py build --target aarch64-unknown-nto-qnx710,x86_64-pc-nto-qnx710,${NATIVE_TARGET} rustc library/core library/alloc library/std library tools/rust-analyzer tools/rustfmt tools/clippy tools/miri

# === Install Rust toolchain ===
echo "[*] Installing Rust toolchain..."
mkdir -p "$INSTALL_DIR/$TOOLCHAIN_NAME"
cp -r build/$NATIVE_TARGET/stage1/* "$INSTALL_DIR/$TOOLCHAIN_NAME/"
cp build/$NATIVE_TARGET/stage1-tools-bin/rust-analyzer "$INSTALL_DIR/$TOOLCHAIN_NAME/bin/rust-analyzer"

# === Register toolchain with rustup ===
echo "[*] Registering toolchain with rustup as '$TOOLCHAIN_NAME'..."
rustup toolchain link "$TOOLCHAIN_NAME" "$INSTALL_DIR/$TOOLCHAIN_NAME"

# === Remove build artifacts ===
cd ..
rm -rf $SRC_DIR
echo "[✓] Rust toolchain for QNX 7.1 installed at: $INSTALL_DIR"
