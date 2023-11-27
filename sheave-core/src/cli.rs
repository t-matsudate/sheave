//! # The common CLI options.
//!
//! Both of the client and the server start themselves via CLI.
//! Currently, there are following common options.
//!
//! * `--protocol`
//! * `-h`, `--host`
//! * `-p`, `--port`
//!
//! ## Protocol
//!
//! Currently, following protocols are available.
//!
//! * RTMP(Real-Time Messaging Protocol)
//!
//! `--protocol rtmp` becomes `Protocol::Rtmp`.
//!
//! Values are case insensitive (Option names are lowercase only).
//! We won't implement RTMPE(RTMP *encrypted*) since that has the MITM vulnerability.
//!
//! ## Host
//!
//! A string which can consider as a destination URI.
//! `-h uri` or `--host uri`.
//!
//! ## Port
//!
//! A 16 bits unsigned integer.
//!
//! `-p portnum` or `--port portnum`.

mod protocols;

pub use self::protocols::*;
