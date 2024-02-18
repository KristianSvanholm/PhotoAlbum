FROM rust:1.76 as builder

WORKDIR /build

RUN apt-get update && apt-get install -y musl-tools openssl libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown

RUN cargo install cargo-leptos cargo-generate

COPY . .

RUN cargo leptos build --release

FROM debian:bookworm

WORKDIR /app

COPY --from=builder /build/target/release/photo-album ./photo-album
COPY --from=builder /build/target/site ./site

ENV LEPTOS_OUTPUT_NAME="photo-album"
ENV LEPTOS_SITE_ROOT="/app/site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"

EXPOSE 3000

CMD ["./photo-album"]
