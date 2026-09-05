#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

REPO=".ipfs"
KUBO_BIN=""
API_PORT=5001
TRUNK_PORT=8080

# --- find free port ---
find_free_port() {
    local port="$1"
    while port_in_use "$port"; do
        port=$((port + 1))
    done
    echo "$port"
}

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
    local port="$1"
    local origins="[\"http://localhost:$port\",\"http://127.0.0.1:$port\",\"http://[::1]:$port\"]"
    local current
    current=$(IPFS_PATH="$REPO" "$KUBO_BIN" config API.HTTPHeaders.Access-Control-Allow-Origin 2>/dev/null || echo "null")
    if [ "$current" != "$origins" ]; then
        echo "Configuring CORS for WASM dashboard on port $port ..."
        IPFS_PATH="$REPO" "$KUBO_BIN" config --json API.HTTPHeaders.Access-Control-Allow-Origin "$origins"
        IPFS_PATH="$REPO" "$KUBO_BIN" config --json API.HTTPHeaders.Access-Control-Allow-Methods '["PUT","POST","GET"]' || true
        CORS_CHANGED=1
    else
        CORS_CHANGED=0
    fi
}

# --- restart daemon ---
restart_daemon() {
    echo "Restarting Kubo daemon to apply new CORS config ..."
    if command -v lsof >/dev/null 2>&1; then
        local pids
        pids=$(lsof -ti ":$API_PORT" 2>/dev/null || true)
        if [ -n "$pids" ]; then
            # shellcheck disable=SC2086
            kill $pids 2>/dev/null || true
        fi
    fi
    # Wait up to 15s for port to be fully released
    for i in $(seq 1 30); do
        if ! port_in_use "$API_PORT"; then
            break
        fi
        sleep 0.5
    done
    # Force-kill anything still holding the port
    if port_in_use "$API_PORT" && command -v lsof >/dev/null 2>&1; then
        local pids
        pids=$(lsof -ti ":$API_PORT" 2>/dev/null || true)
        if [ -n "$pids" ]; then
            # shellcheck disable=SC2086
            kill -9 $pids 2>/dev/null || true
            sleep 1
        fi
    fi
    start_daemon
    if ! port_in_use "$API_PORT"; then
        echo "Error: daemon failed to start after restart."
        exit 1
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

TRUNK_PORT=$(find_free_port "$TRUNK_PORT")
configure_cors "$TRUNK_PORT"

if [ "${CORS_CHANGED:-0}" = "1" ] && port_in_use "$API_PORT"; then
    restart_daemon
else
    start_daemon
fi

echo "Starting WASM dashboard at http://localhost:$TRUNK_PORT"
echo ""
cd examples/wasm-dashboard
exec env -u NO_COLOR trunk serve --port "$TRUNK_PORT"
