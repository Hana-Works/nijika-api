# Build Stage
FROM rust:1.85-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y\ 
    pkg-config\ 
    libssl-dev\ 
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y\
    libssl3\ 
    ca-certificates\
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/nijika-api /app/nijika-api

# Expose the port the app runs on
EXPOSE 3000

# Set the entrypoint
CMD ["/app/nijika-api"]
