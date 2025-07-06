FROM rust:1.88.0
ADD . /sheave
WORKDIR /sheave
ENV FEATURES="sqlite"
ENV TOPIC_DATABASE_URL="sqlite:/tmp/sheave/sheave.db?mode=rwc"
ENV TOPIC_STORAGE_PATH="/tmp/sheave"
ENV LISTENERS="rtmp://0.0.0.0:1935/ondemand"
RUN cargo install sqlx-cli
RUN mkdir -p $TOPIC_STORAGE_PATH/ondemand
RUN DATABASE_URL=$TOPIC_DATABASE_URL sqlx migrate run --source ./resources/migrations
RUN cargo update && cargo build -p sheave-server --features $FEATURES --release --bins
CMD ./target/release/sheave-server
