# Sheave

The RTMP server/client written by Rust.

## Usage

### Server

* Just on this repository

1. `git clone https://github.com/t-matsudate/sheave`
2. `cd ./sheave/sheave-server`
3. `cargo run -- --protocol rtmp -a 127.0.0.1 -p 1935`

* On the docker image

`docker run --rm -it -e PROTOCOL=rtmp -e HOST=127.0.0.1 -e PORT=1935 tmatsudate/sheave-server:latest`

## Documentation

* [Core library](https://t-matsudate.github.io/sheave/sheave_core)

## Goals

The third choice for personal use of RTMP tools.

* Available to be free in common features.
* Open Source and extensible.
* Easy to use for any small use cases. (e.g. on the VPS)

## Features

- [x] RTMP handshake.
- [ ] Configuration exchange.

## License

MIT License
Copyright (c) 2023 Tsuyoshi Matsudate
