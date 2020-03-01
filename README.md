# sheave

An RTMP Server implementation for Rust.

![Testing](https://github.com/t-matsudate/sheave/workflows/Testing/badge.svg?branch=v0.x)

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

[GitHub](https://t-matsudate.github.io/sheave/sheave)

## Abilities

- [x] Doing RTMP's handshake.
- [x] Hnadling the invocations from the client.
- [x] Storing the audio/video data to be sent from the client.
