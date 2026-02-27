#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "Usage: $0 <service> <host> <env-file>"
    echo ""
    echo "Arguments:"
    echo "  service   Cargo package name (e.g. game-server)"
    echo "  host      SSH host (e.g. steve@myserver)"
    echo "  env-file  Path to .env file"
    echo ""
    echo "Example:"
    echo "  $0 game-server steve@myserver game-server.env"
    exit 1
}

if [ $# -lt 3 ]; then
    usage
fi

SERVICE="$1"
HOST="$2"
ENV_FILE="$3"

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
UNIT_TEMPLATE="$REPO_ROOT/infra/systemd/app.service"

# Derive the binary name (cargo uses underscores in binary names)
BINARY_NAME="$(echo "$SERVICE" | tr '-' '_')"

echo "==> Building $SERVICE in release mode..."
cargo build --release -p "$SERVICE" --manifest-path "$REPO_ROOT/Cargo.toml"

BINARY_PATH="$REPO_ROOT/target/release/$BINARY_NAME"
if [ ! -f "$BINARY_PATH" ]; then
    echo "ERROR: Binary not found at $BINARY_PATH"
    exit 1
fi

echo "==> Deploying to $HOST..."

# Create directories on server
ssh "$HOST" "sudo mkdir -p /etc/$SERVICE && sudo mkdir -p /usr/local/bin"

# Copy binary
echo "  Copying binary..."
scp "$BINARY_PATH" "$HOST:/tmp/$BINARY_NAME"
ssh "$HOST" "sudo mv /tmp/$BINARY_NAME /usr/local/bin/$BINARY_NAME && sudo chmod +x /usr/local/bin/$BINARY_NAME"

# Copy env file
echo "  Copying environment file..."
scp "$ENV_FILE" "$HOST:/tmp/$SERVICE.env"
ssh "$HOST" "sudo mv /tmp/$SERVICE.env /etc/$SERVICE/.env && sudo chmod 600 /etc/$SERVICE/.env"

# Install systemd unit (substitute %i with service name locally, then copy)
echo "  Installing systemd unit..."
UNIT_FILE="$(mktemp)"
trap 'rm -f "$UNIT_FILE"' EXIT
sed "s/%i/$SERVICE/g" "$UNIT_TEMPLATE" > "$UNIT_FILE"
scp "$UNIT_FILE" "$HOST:/tmp/$SERVICE.service"
ssh "$HOST" "sudo mv /tmp/$SERVICE.service /etc/systemd/system/$SERVICE.service"

# Reload and restart
echo "  Restarting service..."
ssh "$HOST" "sudo systemctl daemon-reload"
ssh "$HOST" "sudo systemctl enable $SERVICE"
ssh "$HOST" "sudo systemctl restart $SERVICE"

echo "==> Deployed $SERVICE to $HOST"
echo "    Check status: ssh $HOST sudo systemctl status $SERVICE"
echo "    View logs:    ssh $HOST sudo journalctl -u $SERVICE -f"
