FROM rust as builder

RUN apt-get update && apt-get install -y musl-tools musl-dev protobuf-compiler
RUN rustup target add x86_64-unknown-linux-musl && rustup component add clippy

WORKDIR /usr/src/gh_runner_uptime
COPY ./Cargo.toml ./Cargo.toml

# only build dependencies to cache them
COPY ./src/dummy.rs ./src/dummy.rs
RUN sed -i 's#src/main.rs#src/dummy.rs#' Cargo.toml && \
    cargo build --target x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    sed -i 's#src/dummy.rs#src/main.rs#' Cargo.toml
# now copy actual source
COPY ./src ./src

RUN cargo clippy -- -D warnings && \
    cargo test && \
    cargo test --release && \
    cargo build --release --target x86_64-unknown-linux-musl

FROM alpine
COPY --from=builder /usr/src/gh_runner_uptime/target/x86_64-unknown-linux-musl/release/gh_runner_uptime /gh_runner_uptime

WORKDIR /app
ENTRYPOINT ["/gh_runner_uptime"]

LABEL org.opencontainers.image.source=https://github.ibmgcloud.net/dth/gh_runner_uptime
