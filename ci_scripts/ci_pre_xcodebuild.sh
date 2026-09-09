#!/bin/bash
set -euo pipefail

echo "=== Xcode Cloud pre-xcodebuild setup ==="

# Ensure Cargo/Rustup are available for the Xcode build phases.
CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
if [ -f "$CARGO_HOME/env" ]; then
    # shellcheck source=/dev/null
    source "$CARGO_HOME/env"
fi

# Ensure Go is available for the Rust build script (build.rs).
GO_INSTALL_DIR="$HOME/go-install"
if [ -x "$GO_INSTALL_DIR/bin/go" ]; then
    export GO="$GO_INSTALL_DIR/bin/go"
    export PATH="$GO_INSTALL_DIR/bin:$PATH"
fi

# In Xcode Cloud the repo is checked out to the working directory.
# Build the Rust XCFramework here so it is already present when Xcode
# resolves Swift Package Manager dependencies (which happens before
# any build-phase scripts run).
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_SCRIPT="$REPO_ROOT/xcode/build.sh"

if [ -x "$BUILD_SCRIPT" ]; then
    echo "Building Rust XCFramework for SPM..."
    (cd "$REPO_ROOT/xcode" && "$BUILD_SCRIPT")
else
    echo "Build script not found: $BUILD_SCRIPT" >&2
    exit 1
fi

# Verify the XCFramework that SPM expects is in place.
XCFRAMEWORK="$REPO_ROOT/xcode/swiftyapp/Lib/swiftyrustlib/artifacts/RustyCore.xcframework"
if [ ! -d "$XCFRAMEWORK" ]; then
    echo "XCFramework missing after build: $XCFRAMEWORK" >&2
    exit 1
fi

echo "XCFramework ready: $XCFRAMEWORK"
echo "=== Pre-xcodebuild setup complete ==="
