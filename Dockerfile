FROM rust:1.74.1
ADD . /sheave
WORKDIR /sheave
ENV PROTOCOL=rtmp
ENV HOST=127.0.0.1
ENV PORT=1935
RUN cargo update && cargo build -p sheave-server --release --bins
CMD ./target/release/sheave-server --protocol ${PROTOCOL} -a ${HOST} -p ${PORT}
