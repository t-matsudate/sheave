//! # The Chunk Headers
//!
//! Every chunk has following headers:
//!
//! 1. [The Basic Header]
//! 2. [The Message Header]
//! 3. [Extended Timestamp]
//!
//! ## [The Basic Header]
//!
//! This indicates:
//!
//! * A pattern of followed message header.
//! * Which the chunk stream are we in.
//!
//! ## [The Message Header]
//!
//! This indicates:
//!
//! * A timestamp which has passed from its stream started.
//! * A pattern of followed chunk data.
//! * Which the message stream are we in.
//!
//! This must be depended its format on a value in the basic header.
//! That is, the message header has following correspondence with the basic header:
//!
//! |Number|Expected Format|
//! | -: | -: |
//! |`0`|11 bytes|
//! |`1`|7 bytes|
//! |`2`|3 bytes|
//! |`3`|0 bytes|
//!
//! ## Extended Timestamp
//!
//! This is added when a timestamp in its message header exceeded the 3 bytes range.
//! In that case, note its field must be filled with `0xFFFFFF (16777215)`.
//!
//! [The Basic Header]: BasicHeader
//! [The Message Header]: MessageHeader

mod basic;
mod message;

pub use self::{
    basic::{
        BasicHeader,
        MessageFormat
    },
    message::{
        MessageType,
        MessageHeader,
        New,
        SameSource,
        TimerChange
    }
};
