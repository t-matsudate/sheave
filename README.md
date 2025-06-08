# Sheave

The RTMP server/client written by Rust.

## Usage

### Server

* Just on this repository

1. `git clone https://github.com/t-matsudate/sheave`
2. `cd ./sheave/sheave-server`
3. `cargo run -- --topic-database-url sqlite::memory: --listeners rtmp://127.0.0.1:1935/ondemand`

* On the docker image

`docker run --rm -it -e TOPIC_DATABASE_URL=sqlite::memory: -e LISTENERS=rtmp://127.0.0.1:1935/ondemand tmatsudate/sheave-server:latest`

### Client(as a publisher)

* Just on this repository

1. `git clone https://github.com/t-matsudate/sheave`
2. `cd ./sheave/sheave-client`
3. `cargo run -- --client-type publisher -f flv -i filename.flv -f flv rtmp://127.0.0.1:1935/ondemand`

### Client(as a subscriber)

* Just on this repository

1. `git clone https://github.com/t-matsudate/sheave`
2. `cd ./sheave/sheave-client`
3. `cargo run -- --client-type subscriber -f flv -o filename.flv -f flv rtmp://127.0.0.1:1935/ondemand`

## Documentation

* [Core library](https://t-matsudate.github.io/sheave/sheave_core)
* [Server](https://t-matsudate.github.io/sheave/sheave_server)
* [Client](https://t-matsudate.github.io/sheave/sheave_client)

## Goals

The third choice for personal use of RTMP tools.

* Available to be free in common features.
* Open Source and extensible.
* Easy to use for any small use cases. (e.g. on the VPS)

## Features

- [x] RTMP handshake.
- [x] Configuration exchange.
- [x] Storing/Publishing audio/video data.
- [x] Outputting in detail by loggers.
- [x] Subscribing audio/video data.
- [ ] Encoding/Decoding audio/video data.

## License

MIT License
Copyright (c) 2023 Tsuyoshi Matsudate
