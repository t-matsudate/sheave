FROM rust:1.89.0
ADD . /sheave
WORKDIR /sheave
ENV TMPDIR="/tmp"
ENV FEATURES="sqlite"
ENV MIGRATIONS_PATH="resources/migrations"
ENV TOPIC_DATABASE_URL="sqlite:/tmp/sheave/sheave.db?mode=rwc"
ENV TOPIC_STORAGE_PATH="/tmp/sheave"
ENV LISTENERS="rtmp://0.0.0.0:1935/ondemand"
RUN mkdir -p $TOPIC_STORAGE_PATH/ondemand
RUN cargo update && cargo build -p sheave-server --features $FEATURES --release --bins
CMD ./target/release/sheave-server
