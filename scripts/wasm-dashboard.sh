#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

REPO=".ipfs"
KUBO_BIN=""
API_PORT=5001
TRUNK_PORT=8080

# --- find kubo binary ---
find_kubo() {
    if [ -x "go/kubo-sys/cmd/ipfs/ipfs" ]; then
        KUBO_BIN="go/kubo-sys/cmd/ipfs/ipfs"
        return
    fi
    if command -v ipfs >/dev/null 2>&1; then
        KUBO_BIN="ipfs"
        return
    fi
    echo "Error: no Kubo (ipfs) binary found."
    echo "  Build it:  make build-go"
    echo "  Or install ipfs from https://docs.ipfs.tech/install/"
    exit 1
}

# --- check rust target ---
check_wasm_target() {
    if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
        echo "Installing wasm32-unknown-unknown target..."
        rustup target add wasm32-unknown-unknown
    fi
}

# --- check trunk ---
check_trunk() {
    if ! command -v trunk >/dev/null 2>&1; then
        echo "Installing trunk..."
        cargo install trunk
    fi
}

# --- port detection ---
port_in_use() {
    local port="$1"
    if command -v lsof >/dev/null 2>&1; then
        lsof -i ":$port" >/dev/null 2>&1
    elif command -v ss >/dev/null 2>&1; then
        ss -tln | grep -q ":$port "
    elif command -v netstat >/dev/null 2>&1; then
        netstat -tln 2>/dev/null | grep -q ".$port "
    else
        # fallback: try curl
        curl -s -o /dev/null "http://127.0.0.1:$port" || true
        # can't reliably detect, assume free
        return 1
    fi
}

# --- init repo ---
init_repo() {
    if [ ! -f "$REPO/config" ]; then
        echo "Initializing IPFS repo at $REPO ..."
        IPFS_PATH="$REPO" "$KUBO_BIN" init
    fi
}

# --- configure CORS ---
configure_cors() {
    local origins='["http://localhost:8080","http://127.0.0.1:8080","http://[::1]:8080"]'
    local current
    current=$(IPFS_PATH="$REPO" "$KUBO_BIN" config API.HTTPHeaders.Access-Control-Allow-Origin 2>/dev/null || echo "null")
    if [ "$current" != "$origins" ]; then
        echo "Configuring CORS for WASM dashboard..."
        IPFS_PATH="$REPO" "$KUBO_BIN" config --json API.HTTPHeaders.Access-Control-Allow-Origin "$origins"
        IPFS_PATH="$REPO" "$KUBO_BIN" config --json API.HTTPHeaders.Access-Control-Allow-Methods '["PUT","POST","GET"]' || true
    fi
}

# --- start daemon ---
start_daemon() {
    if port_in_use "$API_PORT"; then
        echo "Port $API_PORT is already in use (Kubo API likely running)."
        return
    fi

    echo "Starting Kubo daemon on port $API_PORT ..."
    IPFS_PATH="$REPO" "$KUBO_BIN" daemon --api "/ip4/127.0.0.1/tcp/$API_PORT" >/dev/null 2>&1 &
    local pid=$!

    # wait for API to come up
    for i in $(seq 1 30); do
        if curl -s -o /dev/null "http://127.0.0.1:$API_PORT/api/v0/id" 2>/dev/null; then
            echo "Daemon ready (PID $pid)."
            return
        fi
        sleep 0.5
    done

    echo "Error: daemon did not start within 15 seconds."
    kill "$pid" 2>/dev/null || true
    exit 1
}

# --- main ---
BUILD_ONLY=false
if [ "${1:-}" = "--build-only" ]; then
    BUILD_ONLY=true
fi

find_kubo
check_wasm_target
check_trunk

if [ "$BUILD_ONLY" = true ]; then
    cd examples/wasm-dashboard
    exec env -u NO_COLOR trunk build
fi

init_repo
configure_cors
start_daemon

if port_in_use "$TRUNK_PORT"; then
    echo ""
    echo "Warning: port $TRUNK_PORT is already in use."
    echo "  trunk serve may fail with 'Address already in use'."
    echo "  Kill the existing process or use a different port:"
    echo "    cd examples/wasm-dashboard && trunk serve --port 8081"
    echo ""
fi

echo "Starting WASM dashboard at http://localhost:$TRUNK_PORT"
echo ""
cd examples/wasm-dashboard
exec env -u NO_COLOR trunk serve
