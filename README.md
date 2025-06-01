# Sheave

The RTMP server/client written by Rust.

## Usage

### Server

* Just on this repository

1. `git clone https://github.com/t-matsudate/sheave`
2. `cd ./sheave/sheave-server`
3. `cargo run -- --rtmp 127.0.0.1:1935`

* On the docker image

`docker run --rm -it -e PROTOCOL=rtmp -e HOST=127.0.0.1 -e PORT=1935 tmatsudate/sheave-server:latest`

### Client

* Just on this repository

1. `git clone https://github.com/t-matsudate/sheave`
2. `cd ./sheave/sheave-client`
3. `cargo run -- -i filename.flv -f flv rtmp://127.0.0.1:1935`

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
