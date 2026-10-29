# Image ancord-node: node (Rust) + cloudflared (ARCHITECTURE.md §12.5). Sestavení: docker build -t ancord-node node/
# Kontejner nepublikuje žádný port; dosažitelnost je jen přes tunel, který node spustí s tokenem od Workeru.

FROM rust:1.99-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p ancord-node -p ancord-cli \
 && install -d /out && install -m 0755 target/release/ancord-node target/release/ancord /out/

FROM debian:bookworm-slim AS cloudflared
# Pevná verze a SHA-256 z release notes (https://github.com/cloudflare/cloudflared/releases).
ARG CLOUDFLARED_VERSION=2026.10.3
ARG CLOUDFLARED_SHA256_AMD64=053b80b0a23031a038fdbf3c073f59f82b8f1f152aa67971d7383ffafc0dbab0
ARG CLOUDFLARED_SHA256_ARM64=89297ebfb9e8c99c0dc013a1140601453cf3798d8306c5591651f4a3f5210be8
ARG TARGETARCH
ADD https://github.com/cloudflare/cloudflared/releases/download/${CLOUDFLARED_VERSION}/cloudflared-linux-${TARGETARCH} /cloudflared
RUN case "${TARGETARCH}" in \
      amd64) sum="${CLOUDFLARED_SHA256_AMD64}" ;; \
      arm64) sum="${CLOUDFLARED_SHA256_ARM64}" ;; \
      *) echo "unsupported arch ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
 && echo "${sum}  /cloudflared" | sha256sum -c - \
 && chmod 0755 /cloudflared

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates tini \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --home-dir /data --shell /usr/sbin/nologin ancord \
 && install -d -o ancord -g ancord -m 0700 /data
COPY --from=cloudflared /cloudflared /usr/local/bin/cloudflared
COPY --from=build /out/ancord-node /out/ancord /usr/local/bin/

USER ancord
WORKDIR /data
ENV DATA_DIR=/data \
    LISTEN=0.0.0.0:8080 \
    CLOUDFLARED_BIN=/usr/local/bin/cloudflared \
    RUST_LOG=info
VOLUME ["/data"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD ["bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/8080 && printf 'GET /health HTTP/1.0\\r\\n\\r\\n' >&3 && grep -q '\"ok\":true' <&3"]
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/ancord-node"]
