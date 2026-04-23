# ── Stage 1: Builder ──────────────────────────────────────────────────────────
FROM rust:1.78-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies first
COPY Cargo.toml ./
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release --bin upload_service || true
RUN rm -f src/main.rs

# Build the real source
COPY src ./src
RUN touch src/main.rs && cargo build --release --bin upload_service

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates ffmpeg && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/upload_service ./upload_service

RUN mkdir -p /media

EXPOSE 3007

CMD ["./upload_service"]
