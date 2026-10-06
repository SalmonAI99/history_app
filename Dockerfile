FROM rust:1-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential ca-certificates libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown \
    && cargo install --locked --version 0.3.9 cargo-leptos

WORKDIR /app

# Fetch dependencies in a separate layer so source edits do not invalidate the
# registry cache inside this image layer.
COPY Cargo.toml Cargo.lock ./
# Cargo validates declared targets even for `cargo fetch`. Create temporary
# targets for this cache-only step; the real source replaces them below.
RUN mkdir -p src \
    && printf 'pub fn docker_dependency_fetch_stub() {}\n' > src/lib.rs \
    && printf 'fn main() {}\n' > src/main.rs \
    && cargo fetch --locked \
    && rm -rf src

COPY . .

ENV LEPTOS_SITE_ADDR=0.0.0.0:3000 \
    LEPTOS_OUTPUT_NAME=bookreader \
    LEPTOS_SITE_ROOT=target/site

RUN cargo leptos build --release --cargo-locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home --home-dir /app app \
    && mkdir -p /app/target/site /app/data /app/storage /app/content/slides \
    && chown -R app:app /app

WORKDIR /app

COPY --from=builder --chown=app:app /app/target/release/bookreader /app/bookreader
COPY --from=builder --chown=app:app /app/target/site /app/target/site

ENV LEPTOS_SITE_ADDR=0.0.0.0:3000 \
    LEPTOS_OUTPUT_NAME=bookreader \
    LEPTOS_SITE_ROOT=target/site \
    DATABASE_URL=sqlite:///app/data/bookreader.db?mode=rwc \
    UPLOAD_DIR=/app/storage \
    RUST_LOG=info,tower_http=debug

EXPOSE 3000

USER app

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:3000/ > /dev/null || exit 1

CMD ["/app/bookreader"]
