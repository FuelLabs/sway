FROM rust:1.56 as builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    clang \
    libclang-dev \
    libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build/

COPY . .

RUN cargo install forc
