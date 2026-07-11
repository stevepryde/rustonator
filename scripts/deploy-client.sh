#!/usr/bin/env bash
set -euo pipefail

# Build the Rustonator client and deploy it to Cloudflare Pages production.

project="${RUSTONATOR_CLOUDFLARE_PROJECT:-rustonator}"
branch="${RUSTONATOR_CLOUDFLARE_BRANCH:-main}"

# Public game/scores endpoints baked into the client bundle.
game_server="${VITE_GAME_SERVER:-xp-api.stevepryde.com}"
scores_url="${VITE_SCORES_URL:-https://${game_server}}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
client_dir="${repo_root}/client"

cd "${client_dir}"

if ! bunx wrangler whoami >/dev/null 2>&1; then
  echo "Wrangler is not logged in. Run 'bunx wrangler login' locally, then retry." >&2
  exit 1
fi

bun install --frozen-lockfile

VITE_GAME_SERVER="${game_server}" \
VITE_SCORES_URL="${scores_url}" \
VITE_GAME_SERVER_SSL=1 \
bun run build

bunx wrangler pages deploy dist/ --project-name="${project}" --branch="${branch}"
