#!/bin/bash
set -euo pipefail

echo "=== Xcode Cloud post-clone setup ==="

# ------------------------------------------------------------------
# 1. Ensure git submodules are present (Xcode Cloud should clone
#    them automatically when configured, but belt-and-suspenders).
# ------------------------------------------------------------------
if [ ! -f "go/kubo-sys/go.mod" ]; then
    echo "Submodules missing – initializing..."
    git submodule update --init --recursive
fi

# ------------------------------------------------------------------
# 2. Install Go (Xcode Cloud macOS runners do not ship Go).
# ------------------------------------------------------------------
GO_VERSION="1.26.5"
GO_INSTALL_DIR="$HOME/go-install"
GO_BINARY="$GO_INSTALL_DIR/bin/go"

if [ -x "$GO_BINARY" ]; then
    INSTALLED_VERSION=$("$GO_BINARY" version | awk '{print $3}')
    if [ "$INSTALLED_VERSION" = "go$GO_VERSION" ]; then
        echo "Go $GO_VERSION already installed."
    else
        echo "Go version mismatch ($INSTALLED_VERSION), reinstalling..."
        rm -rf "$GO_INSTALL_DIR"
        install_go=1
    fi
else
    install_go=1
fi

if [ "${install_go:-0}" = "1" ]; then
    echo "Installing Go $GO_VERSION..."
    mkdir -p "$GO_INSTALL_DIR"
    curl -fsSL "https://go.dev/dl/go${GO_VERSION}.darwin-arm64.tar.gz" | tar -C "$GO_INSTALL_DIR" --strip-components=1 -xz
    echo "Go installed at: $GO_BINARY"
fi

export PATH="$GO_INSTALL_DIR/bin:$PATH"
echo "Go version: $(go version)"

# ------------------------------------------------------------------
# 3. Install Rust (Xcode Cloud macOS runners do not ship Rust).
# ------------------------------------------------------------------
CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"

if [ -x "$CARGO_HOME/bin/cargo" ]; then
    echo "Rust already installed."
else
    echo "Installing Rust via rustup..."
    export CARGO_HOME
    export RUSTUP_HOME
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --component rustfmt clippy
fi

# Source cargo env so subsequent commands see rustup/cargo.
# shellcheck source=/dev/null
source "$CARGO_HOME/env"

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"

# ------------------------------------------------------------------
# 4. Install required Rust targets for iOS / Mac Catalyst builds.
# ------------------------------------------------------------------
echo "Installing Rust targets..."
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add aarch64-apple-ios-macabi

# ------------------------------------------------------------------
# 5. Build the Rust XCFramework so it exists before Xcode Cloud
#    resolves Swift Package Manager dependencies.
# ------------------------------------------------------------------
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_SCRIPT="$REPO_ROOT/xcode/build.sh"

if [ -x "$BUILD_SCRIPT" ]; then
    echo ""
    echo "=== Building Rust XCFramework for SPM ==="
    # Ensure build.rs can find Go even though build.sh also checks.
    export GO="$GO_INSTALL_DIR/bin/go"
    (cd "$REPO_ROOT/xcode" && "$BUILD_SCRIPT")
else
    echo "Build script not found: $BUILD_SCRIPT" >&2
    exit 1
fi

XCFRAMEWORK="$REPO_ROOT/xcode/swiftyapp/Lib/swiftyrustlib/artifacts/RustyCore.xcframework"
if [ ! -d "$XCFRAMEWORK" ]; then
    echo "XCFramework missing after build: $XCFRAMEWORK" >&2
    exit 1
fi

echo "XCFramework ready: $XCFRAMEWORK"

# ------------------------------------------------------------------
# 6. Verify everything is on PATH for the Xcode build phases.
# ------------------------------------------------------------------
echo ""
echo "=== Verification ==="
echo "go:    $(which go)"
echo "cargo: $(which cargo)"
echo ""
echo "=== Xcode Cloud post-clone setup complete ==="
