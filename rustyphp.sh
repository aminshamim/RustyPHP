#!/usr/bin/env bash
# RustyPHP convenience launcher
#
# Features:
#  - Kills any process already listening on the chosen port (default 8080)
#  - Starts the php-web playground server (debug or --release)
#  - Waits for readiness, then opens the browser (unless --no-open)
#  - Simple arg parsing: --release, --port <n>, --no-open, --detach
#
# Usage:
#   ./rustyphp.sh                # run debug build on :8080 and open browser
#   ./rustyphp.sh --release      # run release build
#   ./rustyphp.sh --port 9001    # custom port
#   ./rustyphp.sh --no-open      # do not open browser
#   ./rustyphp.sh --detach       # do not attach to server logs (prints PID)
#
set -euo pipefail

PORT=10101
OPEN_BROWSER=1
RELEASE=0
DETACH=0
CARGO_CMD=(cargo run -p php-web --bin server)

# Simple arg parsing
while [[ $# -gt 0 ]]; do
  case "$1" in
    --release)
      RELEASE=1
      shift
      ;;
    --port)
      PORT=${2:?Missing port value}
      shift 2
      ;;
    --no-open)
      OPEN_BROWSER=0
      shift
      ;;
    --detach)
      DETACH=1
      shift
      ;;
    -h|--help)
      grep '^# ' "$0" | sed 's/^# //' | sed '1d'
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ ${RELEASE} -eq 1 ]]; then
  CARGO_CMD=(cargo run --release -p php-web --bin server)
fi

# Resolve project root (directory of this script)
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "$SCRIPT_DIR"

BOLD='\033[1m'
GREEN='\033[32m'
YELLOW='\033[33m'
RED='\033[31m'
NC='\033[0m'

info() { echo -e "${BOLD}${GREEN}[RustyPHP]${NC} $*"; }
warn() { echo -e "${BOLD}${YELLOW}[RustyPHP]${NC} $*"; }
error() { echo -e "${BOLD}${RED}[RustyPHP]${NC} $*"; }

# Function to kill existing process on port
kill_port() {
  local port=$1
  local pids
  
  info "Checking for processes on port ${port}..."
  
  if command -v lsof >/dev/null 2>&1; then
    # Get PIDs using lsof
    pids=$(lsof -nP -iTCP:"$port" -sTCP:LISTEN 2>/dev/null | awk 'NR>1 {print $2}' | sort -u || true)
  else
    # fallback using netstat (Linux)
    pids=$(netstat -tulpn 2>/dev/null | grep ":$port " | awk '{print $7}' | cut -d'/' -f1 | sort -u || true)
  fi
  
  if [[ -n "${pids}" && "${pids}" != "" ]]; then
    warn "Port ${port} in use by PID(s): ${pids}. Killing processes..."
    echo "$pids" | while read -r pid; do
      [[ -z "$pid" ]] && continue
      if kill -0 "$pid" 2>/dev/null; then
        info "Killing process $pid"
        kill -TERM "$pid" 2>/dev/null || true
        sleep 0.2
        # Force kill if still alive
        if kill -0 "$pid" 2>/dev/null; then
          warn "Force killing process $pid"
          kill -KILL "$pid" 2>/dev/null || true
        fi
      fi
    done
    
    # Wait a bit for processes to clean up
    sleep 1
    
    # Verify port is free
    if command -v lsof >/dev/null 2>&1; then
      remaining=$(lsof -nP -iTCP:"$port" -sTCP:LISTEN 2>/dev/null | wc -l || echo "0")
      if [[ $remaining -gt 0 ]]; then
        warn "Some processes may still be using port ${port}"
      else
        info "Port ${port} is now free"
      fi
    fi
  else
    info "Port ${port} is available"
  fi
}

kill_port "$PORT"

# Export PORT so server (if configurable later) can read it
export RUSTYPHP_PORT="$PORT"

info "Starting server on http://127.0.0.1:${PORT} (release=${RELEASE})"

# Run server in background; we rely on server binary binding the default port.
# If the server later supports a --port flag, inject it here.
set +e
echo "[RustyPHP] Launching: ${CARGO_CMD[*]}" >&2
"${CARGO_CMD[@]}" &
SERVER_PID=$!
LAUNCH_STATUS=$?
set -e
if [[ $LAUNCH_STATUS -ne 0 ]]; then
  error "Failed to start cargo run (exit $LAUNCH_STATUS)"; exit 1
fi
if ! kill -0 "$SERVER_PID" 2>/dev/null; then
  error "Cargo process not running (PID $SERVER_PID)"; exit 1
fi

# Ensure we clean up if not detached
if [[ $DETACH -eq 0 ]]; then
  cleanup() {
    if kill -0 "$SERVER_PID" 2>/dev/null; then
      warn "Stopping server (PID $SERVER_PID)"
      kill "$SERVER_PID" 2>/dev/null || true
    fi
  }
  trap cleanup EXIT INT TERM
fi

# Wait for readiness (poll socket)
ATTEMPTS=40
SLEEP=0.15
READY=0
for i in $(seq 1 $ATTEMPTS); do
  if curl -s --max-time 0.3 "http://127.0.0.1:${PORT}/" >/dev/null 2>&1; then
    READY=1
    break
  fi
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    error "Server process exited prematurely. Aborting."; 
    wait "$SERVER_PID" || true
    exit 1
  fi
  sleep $SLEEP
done

if [[ $READY -eq 1 ]]; then
  info "Server is ready at: http://127.0.0.1:${PORT}/"
else
  warn "Server not responding after $(( ATTEMPTS * SLEEP ))s; continuing anyway."
fi

if [[ $OPEN_BROWSER -eq 1 ]]; then
  info "Opening browser..."
  if command -v open >/dev/null 2>&1; then
    open "http://127.0.0.1:${PORT}/" || warn "Failed to open browser automatically"
  elif command -v xdg-open >/dev/null 2>&1; then
    xdg-open "http://127.0.0.1:${PORT}/" || warn "Failed to open browser automatically"
  else
    warn "No known browser opener (open/xdg-open) found."
  fi
  if [[ $? -ne 0 ]]; then
    info "Please manually visit: http://127.0.0.1:${PORT}/"
  fi
else
  info "Server URL: http://127.0.0.1:${PORT}/"
fi

if [[ $DETACH -eq 1 ]]; then
  info "Server running in background (PID $SERVER_PID)."
  echo "$SERVER_PID" > .rustyphp_server.pid 2>/dev/null || true
  exit 0
fi

info "Attaching to server output (Ctrl-C to stop)..."
wait "$SERVER_PID" || true
