# Build stage
FROM rust:alpine AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apk add --no-cache musl-dev

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:latest

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libssl3

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/api /app/api

# Expose the port
EXPOSE 3000

# Run the application
CMD ["/app/api"]