FROM rust:1.80.0-alpine AS builder
RUN apk upgrade --no-cache
RUN apk add --update --no-cache \
    pkgconfig \
    musl-dev \
    openssl-dev
WORKDIR /build
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release

FROM alpine:3.20 AS runner
RUN apk upgrade --no-cache
RUN apk add --update --no-cache \
    libgcc \
    openssl
WORKDIR /bot
COPY --from=builder /build/target/release/office-notice ./
CMD ./office-notice