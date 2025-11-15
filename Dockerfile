FROM rust:1.90.0
ADD . /sheave
WORKDIR /sheave
ENV TMPDIR="/tmp"
ENV FEATURES="mysql"
ENV LOGLEVEL="info"
ENV MIGRATIONS_PATH="resources/migrations"
ENV TOPIC_STORAGE_PATH="/tmp/sheave"
ENV LISTENERS="rtmp://0.0.0.0:1935/ondemand"
RUN mkdir -p $TOPIC_STORAGE_PATH/ondemand
RUN cp resources/test.flv $TOPIC_STORAGE_PATH/ondemand
RUN cargo build -p sheave-server --features $FEATURES --release --bins
CMD ./target/release/sheave-server
