# Stage 1 — Build
FROM rust:1.91.1 AS builder
WORKDIR /app

# Copy entire workspace
COPY ../../ ./

# Build only the API binary (workspace lives under /app/backend)
WORKDIR /app/backend
RUN cargo build --release -p mnemo-api-service

# Stage 2 — Runtime
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/backend/target/release/mnemo-api-service .

EXPOSE 7700
CMD ["./mnemo-api-service"]
