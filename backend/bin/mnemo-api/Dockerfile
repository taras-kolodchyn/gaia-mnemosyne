# Stage 0 — Base with build tooling
FROM rust:1.91.1 AS chef
WORKDIR /app/backend
RUN cargo install cargo-chef sccache

ENV RUSTC_WRAPPER=/usr/local/cargo/bin/sccache
ENV SCCACHE_DIR=/sccache
ENV SCCACHE_CACHE_SIZE=5G

# Stage 1 — Plan dependency build
FROM chef AS planner
WORKDIR /app/backend

COPY backend ./

RUN cargo chef prepare --recipe-path recipe.json

# Stage 2 — Build dependencies, then the API binary
FROM chef AS builder
WORKDIR /app/backend

COPY --from=planner /app/backend/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/sccache \
    --mount=type=cache,target=/app/backend/target \
    cargo chef cook --release --recipe-path recipe.json

COPY backend ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/sccache \
    --mount=type=cache,target=/app/backend/target \
    cargo build --release -p mnemo-api-service

# Stage 2 — Runtime
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/backend/target/release/mnemo-api-service .

EXPOSE 7700
CMD ["./mnemo-api-service"]
