FROM rust:1.82.0
ADD . /sheave
WORKDIR /sheave
ENV HOST=127.0.0.1
ENV PORT=1935
RUN cargo update && cargo build -p sheave-server --release --bins
CMD ./target/release/sheave-server --rtmp "${HOST}:${PORT}"
