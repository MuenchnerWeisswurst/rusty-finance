FROM rust:slim-buster as builder
RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown
WORKDIR /usr/src/myapp
COPY . .
RUN cargo build --release --bin server
RUN cd frontend && trunk build --release --public-url /assets/

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*
RUN mkdir /app/
COPY --from=builder /usr/src/myapp/target/release/server /app/server
COPY --from=builder /usr/src/myapp/dist /app/dist

ENV PORT=8080
ENV LISTEN_ADDRESS=0.0.0.0
ENV RUST_LOG=debug
ENV DATABASE_URL=postgres://name:pw@host/db
WORKDIR /app
CMD ["./server"]
