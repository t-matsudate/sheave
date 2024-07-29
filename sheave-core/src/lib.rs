#![allow(unexpected_cfgs)]

pub mod handshake;
pub mod net;
pub mod readers;
pub mod writers;
pub mod messages;
mod decoder;
mod encoder;
mod byte_buffer;
pub mod handlers;
pub mod flv;

pub use self::{
    decoder::Decoder,
    encoder::Encoder,
    byte_buffer::*
};

pub const U24_MAX: u32 = 0x00ffffff;
