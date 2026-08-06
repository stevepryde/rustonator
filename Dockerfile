# syntax=docker/dockerfile:1.7
#
# Multi-stage build for the Rustonator servers.
#
# The builder runs on the *native* build platform and cross-compiles to
# x86_64 with cargo-zigbuild (same approach as soulfire) — building via
# qemu emulation segfaults rustc on Apple Silicon.
#
# Build a specific image with:
#   docker buildx build --platform linux/amd64 --target game-server   -t stevepryde/rustonator-game .
#   docker buildx build --platform linux/amd64 --target scores-server -t stevepryde/rustonator-scores .
#
# The full publish flow is the spdeploy 'publish_images' operation in deploy.yml.

FROM --platform=$BUILDPLATFORM rust:1-bookworm AS builder

ARG BUILDARCH
ARG ZIG_VERSION=0.14.1

RUN apt-get update \
    && apt-get install -y --no-install-recommends xz-utils \
    && rm -rf /var/lib/apt/lists/*

RUN case "${BUILDARCH}" in \
      amd64) zig_arch="x86_64" ;; \
      arm64) zig_arch="aarch64" ;; \
      *) echo "Unsupported BuildKit architecture for Zig: ${BUILDARCH}" >&2; exit 1 ;; \
    esac \
    && curl -fsSL "https://ziglang.org/download/${ZIG_VERSION}/zig-${zig_arch}-linux-${ZIG_VERSION}.tar.xz" -o /tmp/zig.tar.xz \
    && mkdir -p /opt/zig \
    && tar -xJf /tmp/zig.tar.xz --strip-components=1 -C /opt/zig \
    && ln -s /opt/zig/zig /usr/local/bin/zig \
    && rm /tmp/zig.tar.xz

RUN rustup target add x86_64-unknown-linux-gnu
RUN cargo install cargo-zigbuild --locked

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY game-server ./game-server
COPY scores-server ./scores-server

# bookworm ships glibc 2.36
RUN cargo zigbuild --release -p game_server -p scores_server \
    --target x86_64-unknown-linux-gnu.2.36

FROM --platform=linux/amd64 debian:bookworm-slim AS game-server
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/game_server /usr/local/bin/game_server
EXPOSE 9002
CMD ["game_server"]

FROM --platform=linux/amd64 debian:bookworm-slim AS scores-server
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/scores_server /usr/local/bin/scores_server
EXPOSE 9003
CMD ["scores_server"]
