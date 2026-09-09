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

# In Xcode Cloud the build is run from a workspace directory that contains
# the cloned repo.  The Xcode project expects SRCROOT to be relative to the
# workspace root, so nothing special is required here.

echo "=== Environment ==="
echo "PATH: $PATH"
echo "GO:   ${GO:-(not set)}"
echo "=== Pre-xcodebuild setup complete ==="
