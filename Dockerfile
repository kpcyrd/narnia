# syntax=docker/dockerfile:1
FROM rust:alpine3.16
ENV RUSTFLAGS="-C target-feature=-crt-static"
WORKDIR /usr/src/narnia
RUN apk add --no-cache musl-dev make autoconf automake git libtool openssl-dev
COPY . .
RUN --mount=type=cache,target=/var/cache/buildkit \
    CARGO_HOME=/var/cache/buildkit/cargo \
    CARGO_TARGET_DIR=/var/cache/buildkit/target \
    cargo build --release --locked && \
    cp -v /var/cache/buildkit/target/release/narnia /

FROM alpine:3.16
RUN apk add --no-cache libgcc openssl
COPY --from=0 /narnia /usr/local/bin/
ENV NARNIA_DATA_DIR=/data
VOLUME ["/data"]
ENTRYPOINT ["narnia"]
