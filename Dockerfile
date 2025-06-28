# Build stage
FROM rust:1.82-slim AS builder

WORKDIR /usr/src/inventory
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y sqlite3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/inventory/target/release/inventory /usr/local/bin/inventory

# Set up volume for persistent data
VOLUME /root/.inventory

ENTRYPOINT ["inventory"]
CMD ["--help"] 