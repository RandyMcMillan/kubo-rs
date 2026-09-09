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

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
XCFRAMEWORK="$REPO_ROOT/xcode/swiftyapp/Lib/swiftyrustlib/artifacts/RustyCore.xcframework"

# If the XCFramework is missing (shouldn't happen when ci_post_clone.sh
# runs first), rebuild it now as a last resort.
if [ ! -d "$XCFRAMEWORK" ]; then
    echo "XCFramework missing — rebuilding..." >&2
    BUILD_SCRIPT="$REPO_ROOT/xcode/build.sh"
    if [ -x "$BUILD_SCRIPT" ]; then
        (cd "$REPO_ROOT/xcode" && "$BUILD_SCRIPT")
    else
        echo "Build script not found: $BUILD_SCRIPT" >&2
        exit 1
    fi
fi

if [ ! -d "$XCFRAMEWORK" ]; then
    echo "XCFramework still missing after build: $XCFRAMEWORK" >&2
    exit 1
fi

echo "XCFramework ready: $XCFRAMEWORK"
echo "=== Pre-xcodebuild setup complete ==="
