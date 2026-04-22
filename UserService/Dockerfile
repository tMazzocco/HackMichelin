# ── Stage 1: builder ─────────────────────────────────────────────────────────
FROM rust:1.78-slim AS builder

WORKDIR /app

# System deps needed to compile openssl-sys / sqlx
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock* ./

# Create a stub main so `cargo build --release` can cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin user_service || true
RUN rm -rf src

# Copy real source and build
COPY src ./src
RUN touch src/main.rs && cargo build --release --bin user_service

# ── Stage 2: runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/user_service /app/user_service

EXPOSE 3003

CMD ["/app/user_service"]
