# ── Build stage ─────────────────────────────────────────────────────────────
FROM rust:1.78-slim AS builder
WORKDIR /build

# Cache dependencies separately
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release --locked
RUN rm -rf src

# Build actual binary
COPY src ./src
# Touch main.rs so Cargo rebuilds the bin (not just deps)
RUN touch src/main.rs
RUN cargo build --release --locked

# ── Runtime stage ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/maps_data_service .

EXPOSE 3000

CMD ["./maps_data_service"]
