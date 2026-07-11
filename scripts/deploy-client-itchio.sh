#!/usr/bin/env bash
set -euo pipefail

# Build the Rustonator client against the dtn8r instance and push it to
# itch.io with butler.
#
# Requires:
#   - butler on PATH (https://itch.io/docs/butler/installing.html)
#   - BUTLER_API_KEY exported, or a prior 'butler login'

itch_target="${ITCH_IO_TARGET:-inspiredjourneys/detonator}"
itch_channel="${ITCH_IO_CHANNEL:-web}"
game_server="${VITE_GAME_SERVER:-dtn8r-api.inspiredjourneys.dev}"
scores_url="${VITE_SCORES_URL:-https://${game_server}}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
client_dir="${repo_root}/client"

if ! command -v butler >/dev/null 2>&1; then
  echo "butler is not installed. See https://itch.io/docs/butler/installing.html" >&2
  exit 1
fi

version="$(git -C "${repo_root}" rev-parse --short HEAD)"

cd "${client_dir}"

bun install --frozen-lockfile

VITE_GAME_SERVER="${game_server}" \
VITE_SCORES_URL="${scores_url}" \
VITE_GAME_SERVER_SSL=1 \
bun run build

butler push dist/ "${itch_target}:${itch_channel}" --userversion "${version}"
