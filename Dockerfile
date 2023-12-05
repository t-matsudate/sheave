FROM rust:1.74.0
ADD . /sheave
WORKDIR /sheave
RUN cargo update
