# Stage 1: Build
FROM rust:1.81 as builder

# Set work directory
WORKDIR /usr/src/app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "// placeholder" > src/lib.rs
RUN cargo fetch
RUN cargo build --release

# Copy the actual source code
COPY . .
RUN cargo build --release


# Stage 2: Runtime
FROM alpine:3.18

# Set work directory
WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/cryptoquant .

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["./cryptoquant"]