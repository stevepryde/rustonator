#!/usr/bin/env bash
set -euo pipefail

# Build both Rustonator server images for linux/amd64 and push them to
# Docker Hub. Pass a tag as $1 (default: latest). Set DOCKER_OUTPUT=--load
# to build locally without pushing.

image_tag="${RUSTONATOR_IMAGE_TAG:-${1:-latest}}"
docker_output="${DOCKER_OUTPUT:---push}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

docker info >/dev/null

build() {
  local target="$1"
  local image="$2"

  DOCKER_BUILDKIT=1 docker buildx build \
    --platform linux/amd64 \
    --file "${repo_root}/Dockerfile" \
    --target "${target}" \
    --tag "${image}:${image_tag}" \
    "${docker_output}" \
    "${repo_root}"
}

build game-server stevepryde/rustonator-game
build scores-server stevepryde/rustonator-scores
