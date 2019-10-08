# sheave

An RTMP Server implementation for Rust.

[![CircleCI](https://circleci.com/gh/t-matsudate/sheave.svg?style=svg)](https://circleci.com/gh/t-matsudate/sheave)

## Usage

```rust
use std::io::{
    Result as IOResult
};
use sheave::run;

fn main() -> IOResult<()> {
    run()
}
```

## Documentation

[GitHub](https://t-matsudate.github.io/sheave)

## Abilities

- [x] Doing RTMP's handshake.
- [x] Hnadling the invocations from the client.
- [x] Storing the audio/video data to be sent from the client.
