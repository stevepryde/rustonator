#!/usr/bin/env bash
set -euo pipefail

# One-time migration of scores.json from a legacy host to vps-au1.
#
# Usage: migrate-scores.sh <instance> [source-ssh-host]
#
#   instance         rustonator | dtn8r
#   source-ssh-host  defaults: rustonator -> steve@drop1, dtn8r -> steve@soulfire1
#
# Copies /var/lib/scores_server/scores.json from the legacy host into
# /opt/rustonator/data/<instance>/scores.json on vps-au1, then restarts the
# instance's scores container so it picks up the file.

if [ $# -lt 1 ]; then
  echo "Usage: $0 <instance> [source-ssh-host]" >&2
  exit 1
fi

instance="$1"

case "${instance}" in
  rustonator) default_source="steve@drop1" ;;
  dtn8r) default_source="steve@soulfire1" ;;
  *)
    echo "instance must be 'rustonator' or 'dtn8r'" >&2
    exit 1
    ;;
esac

source_host="${2:-${default_source}}"

vps_host="${VPS_HOST:-steve@170.64.177.11}"
remote_source="/var/lib/scores_server/scores.json"
tmp_file="$(mktemp)"
trap 'rm -f "${tmp_file}"' EXIT

echo "==> Fetching ${remote_source} from ${source_host}..."
scp "${source_host}:${remote_source}" "${tmp_file}"

echo "==> Copying to vps-au1..."
scp "${tmp_file}" "${vps_host}:/opt/rustonator/data/${instance}/scores.json"

echo "==> Restarting ${instance}-scores so it loads the migrated file..."
ssh "${vps_host}" "cd /opt/rustonator && docker compose restart ${instance}-scores"

echo "==> Done. Verify with:"
echo "    curl -s https://<instance-domain>/api/scores"
