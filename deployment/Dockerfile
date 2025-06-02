# Stage 1: Build
FROM lukemathwalker/cargo-chef:latest-rust-1.87 as chef
WORKDIR /build/
# hadolint ignore=DL3008
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    lld \
    clang \
    libclang-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /build/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release -p forc --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release -p forc

# Stage 2: Run
FROM ubuntu:20.04 as run

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /root/

COPY --from=builder /build/target/release/forc .

CMD ["exec", "./forc"]
