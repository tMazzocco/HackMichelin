FROM rust:1.78-slim AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml ./
RUN mkdir src && echo "fn main(){}" > src/main.rs && cargo build --release && rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/target/release/feed_service .
EXPOSE 3005
CMD ["./feed_service"]
