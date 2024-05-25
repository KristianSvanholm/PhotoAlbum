FROM rust:latest as builder

WORKDIR /build

RUN apt-get update && apt-get install -y apt-utils musl-tools openssl libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown

RUN cargo install cargo-leptos cargo-generate

COPY .cargo .cargo
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY Makefile.toml Makefile.toml
COPY style style
COPY migrations migrations
COPY src src
COPY public public
COPY model.bin model.bin

RUN cargo leptos build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /build/target/release/photo-album ./photo-album
COPY --from=builder /build/target/site ./site
COPY --from=builder /build/model.bin ./model.bin

ENV LEPTOS_OUTPUT_NAME="photo-album"
ENV LEPTOS_SITE_ROOT="/app/site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"

EXPOSE 3000

CMD ["./photo-album"]
