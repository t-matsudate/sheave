FROM rust:1.74.0
ADD . /sheave
WORKDIR /sheave
ENV PROTOCOL=rtmp
ENV HOST=localhost
ENV PORT=1935
RUN cargo update && cargo build -p sheave-server --release --bins
CMD sheave-server --protocol ${PROTOCOL} -h ${HOST} -p ${PORT}
