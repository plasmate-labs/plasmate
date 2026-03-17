# Plasmate - Agent-native headless browser
# Multi-arch: linux/amd64, linux/arm64
#
# Build with pre-compiled binaries from CI:
#   docker buildx build --platform linux/amd64,linux/arm64 .
#
# Usage:
#   docker run -p 9222:9222 ghcr.io/plasmate-labs/plasmate:latest
#   docker run ghcr.io/plasmate-labs/plasmate:latest fetch https://example.com

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ARG TARGETPLATFORM

# Copy the correct binary based on target platform
COPY docker-build/ /tmp/binaries/
RUN case "$TARGETPLATFORM" in \
      "linux/amd64") cp /tmp/binaries/plasmate-x86_64-linux /usr/local/bin/plasmate ;; \
      "linux/arm64") cp /tmp/binaries/plasmate-aarch64-linux /usr/local/bin/plasmate ;; \
      *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac && \
    chmod +x /usr/local/bin/plasmate && \
    rm -rf /tmp/binaries

RUN useradd -m -s /bin/sh plasmate
USER plasmate

EXPOSE 9222

ENTRYPOINT ["plasmate"]
CMD ["serve", "--host", "0.0.0.0", "--port", "9222"]
