FROM lukemathwalker/cargo-chef:0.1.50-rust-1.66.0 AS chef
WORKDIR /app/
RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown

FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin server --recipe-path recipe.json 

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --bin server --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin server
RUN cd frontend && trunk build --release --public-url /assets/

FROM debian:bullseye AS pg-builder
RUN apt-get update
RUN apt-get install --yes libpq5

FROM gcr.io/distroless/cc-debian11

COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libpq.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libgssapi_krb5.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libldap_r-2.4.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libkrb5.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libk5crypto.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libkrb5support.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/liblber-2.4.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libsasl2.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libgnutls.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libp11-kit.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libidn2.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libunistring.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libtasn1.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libnettle.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libhogweed.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libgmp.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libffi.so* /usr/lib/x86_64-linux-gnu/

COPY --from=pg-builder /lib/x86_64-linux-gnu/libcom_err.so.2 /lib/x86_64-linux-gnu/libcom_err.so.2
COPY --from=pg-builder /lib/x86_64-linux-gnu/libcom_err.so.2.1 /lib/x86_64-linux-gnu/libcom_err.so.2.1
COPY --from=pg-builder /lib/x86_64-linux-gnu/libkeyutils.so.1 /lib/x86_64-linux-gnu/libkeyutils.so.1
COPY --from=pg-builder /lib/x86_64-linux-gnu/libkeyutils.so.1.9 /lib/x86_64-linux-gnu/libkeyutils.so.1.9
WORKDIR /app/
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/dist /app/dist

ENV PORT=8080
ENV LISTEN_ADDRESS=0.0.0.0
ENV RUST_LOG=debug
ENV DATABASE_URL=postgres://name:pw@host/db
WORKDIR /app
CMD ["./server"]
