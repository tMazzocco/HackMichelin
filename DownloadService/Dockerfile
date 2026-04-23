# ── Build stage ───────────────────────────────────────────────────────────────
FROM rust:1.78-slim AS builder
WORKDIR /build

# Cache dependencies separately (layer-cache trick)
COPY Cargo.toml ./
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build actual binary
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/download_service .

EXPOSE 3001

CMD ["./download_service"]
