FROM rust:1.70.0-bookworm as chef
RUN cargo install cargo-chef --locked
WORKDIR /app

RUN apt-get update -y && \
  apt-get install -y --no-install-recommends \
    cmake \
    g++ \
    libsasl2-dev \
    libssl-dev \
    libudev-dev \
    pkg-config \
    protobuf-compiler \
  && \
  rm -rf /var/lib/apt/lists/*

COPY ci/get-protoc.sh ./
RUN chmod +x get-protoc.sh
RUN /app/get-protoc.sh

FROM chef AS planner

COPY Cargo.* ./
COPY app app
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY Cargo.* ./
COPY app app

FROM chef as development
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY Cargo.* ./
COPY app app
RUN cargo install cargo-watch
RUN cargo check --all
CMD ["cargo", "watch", "-x", "run --bin holaplex-hub-analytics"]

FROM builder AS builder-hub-analytics
RUN cargo build --release --bin holaplex-hub-analytics

FROM debian:bookworm-slim as base
WORKDIR /app
RUN apt-get update -y && \
  apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
    libssl1.1 \
  && \
  rm -rf /var/lib/apt/lists/*

FROM base AS hub-analytics
COPY --from=builder-hub-analytics /app/target/release/holaplex-hub-analytics /usr/local/bin
CMD ["/usr/local/bin/holaplex-hub-analytics"]
