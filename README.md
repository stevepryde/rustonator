# Rustonator

Rust port of Detonator (and baseline for future games)

## Deployment

Both game instances (rustonator + dtn8r) run under Docker Compose on vps-au1,
deployed with [spdeploy](https://github.com/stevepryde/spdeploy):

```bash
spdeploy                                   # full deploy: build/push images + vps-au1 runtime
spdeploy --operation deploy_client         # client -> Cloudflare Pages
spdeploy --operation deploy_client_itchio  # client -> itch.io
```

- `deploy.yml` — top-level operations (image publish, client deploys)
- `infra/vps-au1/deploy.yml` — server runtime (env files, compose, Caddy, health checks)
- `scripts/migrate-scores.sh` — one-time scores.json migration from the legacy hosts
