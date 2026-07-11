# Multi-stage build for the Rustonator servers.
#
# Build a specific image with:
#   docker buildx build --target game-server   -t stevepryde/rustonator-game .
#   docker buildx build --target scores-server -t stevepryde/rustonator-scores .
#
# See scripts/publish-rustonator-images.sh for the full publish flow.

FROM rust:1-slim-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY game-server ./game-server
COPY scores-server ./scores-server
RUN cargo build --release -p game_server -p scores_server

FROM debian:bookworm-slim AS game-server
RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/game_server /usr/local/bin/game_server
EXPOSE 9002
CMD ["game_server"]

FROM debian:bookworm-slim AS scores-server
COPY --from=builder /app/target/release/scores_server /usr/local/bin/scores_server
EXPOSE 9003
CMD ["scores_server"]
