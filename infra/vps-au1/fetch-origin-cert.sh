#!/usr/bin/env bash
set -euo pipefail

# Fetch the stevepryde.com Cloudflare origin cert from drop1 into a local
# (gitignored) certs/ dir, so the sync_origin_cert operation can install it
# on vps-au1 for the rustonator Caddy site.

source_host="${ORIGIN_CERT_SOURCE:-steve@drop1}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
certs_dir="${script_dir}/certs"

mkdir -p "${certs_dir}"
scp "${source_host}:/etc/ssl/cloudflare/origin.pem" "${certs_dir}/stevepryde.com-origin-cert.pem"
scp "${source_host}:/etc/ssl/cloudflare/origin-key.pem" "${certs_dir}/stevepryde.com-origin-cert.key"
chmod 600 "${certs_dir}/stevepryde.com-origin-cert.key"
