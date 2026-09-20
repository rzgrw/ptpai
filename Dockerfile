# Stage 1: Build the optimized binary in Rust
FROM rust:1.85-bookworm AS builder

WORKDIR /usr/src/ptpai

# Copy manifest and dependencies
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Compile production release binary with AVX2 and native optimizations
RUN cargo build --release

# Stage 2: Minimal Ubuntu runtime with NVIDIA CUDA GPU compatibility
FROM ubuntu:22.04 AS runner

ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_LOG=info

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libpci3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binary from builder
COPY --from=builder /usr/src/ptpai/target/release/ptpai /usr/local/bin/ptpai

# Expose WebUI & API (TCP 8080) and STUN Holepunch (UDP 8082)
EXPOSE 8080/tcp
EXPOSE 8082/udp

# Healthcheck
HEALTHCHECK --interval=15s --timeout=3s --retries=3 \
    CMD curl -f http://localhost:8080/api/status || exit 1

ENTRYPOINT ["/usr/local/bin/ptpai"]
CMD ["tracker", "--port", "8080"]
