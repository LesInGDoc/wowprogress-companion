FROM rust:alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/target/release/scraper .
CMD ["./scraper"]