FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/plasmate /usr/local/bin/plasmate

EXPOSE 9222

ENTRYPOINT ["plasmate"]
CMD ["serve", "--host", "0.0.0.0", "--port", "9222"]
