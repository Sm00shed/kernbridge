FROM rust:latest

RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add aarch64-unknown-linux-musl

# [1]
RUN useradd -m -u 1001 builder
USER builder

# [2]
WORKDIR /workspace/kernbridge
