# Stage 1: Build
FROM rust:1.81 AS builder

WORKDIR /usr/src/app

# Install required libraries for Rust and OpenSSL
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "// placeholder" > src/lib.rs
RUN cargo build --release --locked

# Copy the actual source code and build
COPY . .
RUN cargo build --release --locked

# Stage 2: Runtime
FROM debian:bookworm

WORKDIR /usr/local/bin

# Install required runtime libraries
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/cryptoquant .

# Set the binary as the entrypoint
ENTRYPOINT ["./cryptoquant"]