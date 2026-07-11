# syntax=docker/dockerfile:1

# Builder and runtime share the same Alpine major so musl matches.
FROM rust:1-alpine3.21 AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./
# Stub sources so cargo can parse [[test]] name = "bdd" while caching deps
RUN mkdir -p src tests \
    && echo "fn main() {}" > src/main.rs \
    && echo "fn main() {}" > tests/bdd.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release \
    && rm -f target/release/deps/priora_api*

COPY migrations ./migrations
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    touch src/main.rs \
    && cargo build --release

# --- runtime (~5–15 MB base + binary; no openssl / libsqlite3) ------------
FROM alpine:3.21

RUN apk add --no-cache ca-certificates \
    && adduser -D -u 10001 -h /home/appuser appuser \
    && mkdir -p /app/data /app/uploads \
    && chown -R appuser:appuser /app

WORKDIR /app

COPY --from=builder /app/target/release/priora-api /app/priora-api
COPY migrations ./migrations

USER appuser

ENV HOST=0.0.0.0 \
    PORT=3000 \
    DATABASE_URL=sqlite:/app/data/priora.db?mode=rwc

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://127.0.0.1:3000/api/health || exit 1

CMD ["./priora-api"]
