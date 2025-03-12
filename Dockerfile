FROM rust as builder

RUN apt-get update && apt-get install -y musl-tools musl-dev protobuf-compiler
RUN rustup target add x86_64-unknown-linux-musl && rustup component add clippy

WORKDIR /usr/src/github_uptime_monitor
COPY ./Cargo.toml ./Cargo.toml

# only build dependencies to cache them
COPY ./src/dummy.rs ./src/dummy.rs
RUN sed -i 's#src/main.rs#src/dummy.rs#' Cargo.toml && \
    cargo clippy -- -D warnings && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    sed -i 's#src/dummy.rs#src/main.rs#' Cargo.toml
# now copy actual source
COPY ./src ./src

RUN cargo clippy -- -D warnings && \
    cargo build --release --target x86_64-unknown-linux-musl

FROM alpine
COPY --from=builder /usr/src/github_uptime_monitor/target/x86_64-unknown-linux-musl/release/github_uptime_monitor /github_uptime_monitor

WORKDIR /app
ENTRYPOINT ["/github_uptime_monitor"]

LABEL org.opencontainers.image.source=https://github.ibmgcloud.net/dth/github_uptime_monitor

