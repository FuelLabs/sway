# Stage 1: Build
FROM rust:1.58.1 as builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    clang \
    libclang-dev \
    libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build/

COPY . .

RUN cargo build --release -p forc

# Stage 2: Run
FROM ubuntu:20.04 as run

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /root/

COPY --from=builder /build/target/release/forc .