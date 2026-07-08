# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/priora_api*

COPY migrations ./migrations
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN useradd --system --uid 10001 --create-home appuser \
    && mkdir -p /app/data /app/uploads \
    && chown -R appuser:appuser /app

COPY --from=builder /app/target/release/priora-api /app/priora-api
COPY migrations ./migrations

USER appuser

ENV HOST=0.0.0.0 \
    PORT=3000 \
    DATABASE_URL=sqlite:/app/data/priora.db?mode=rwc

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:3000/api/health || exit 1

CMD ["./priora-api"]
