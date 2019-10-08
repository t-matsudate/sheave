//! # Real Time Messaging Protocol
//!
//! Mainly, this is the protocol for sending/receiving audio/video data over TCP.
//!
//! ## Protocols
//!
//! As the protocol based on RTMP, there are also following protocols:
//!
//! * RTMPE
//! * RTMPS
//! * RTMPT, RTMPTE, RTMPTS
//!
//! ### RTMPE
//!
//! Real Time Messaging Protocol *encrypted*.
//! This will encrypt the chunk by using Diffie-Hellman key exchange.
//! However this has been deprecated because this hadn't encrypted the network, that is, this can be targetted from man-in-the-middle attack.
//!
//! ### RTMPS
//!
//! Real Time Messaging Protocol over *SSL/TLS*.
//! This will be secure than RTMPE because encrypts the connection with the client.
//!
//! ### RTMPT, RTMPTE, RTMPTS
//!
//! Real Time Messaging Protocol over HTTP/HTTPS.
//! This is the same as RTMP, RTMPE and RTMPS except is on the HTTP/HTTPS.
//!
//! ## Steps
//!
//! RTMP takes following steps:
//!
//! 1. RTMP handshake
//! 2. Invocations
//! 3. Publishing
//!
//! ### RTMP handshake
//!
//! Takes the handshake with the client as RTMP.
//! This can need HMAC-SHA256 digests/signatures at this moment.
//! And if you will use RTMPS/RTMPTS, also the SSL certificate will need.
//! See handshake.rs for more detail about the handshake.
//!
//! ### Invocations
//!
//! Exchanges application information to use, with the client.
//! See handlers.rs for more detail about this phace, and also see messages.rs for more detail about each chunk.
//!
//! ### Publishing
//!
//! Stores audio/video data sent from the client.
//! If some request from other clients has, sends their audio/video data to its client.
//! See handlers.rs for more detail about this phase, and also see flv.rs for more detail about stored audio/video data.
//!
//! [handshake.rs]: ./handshake.rs.html
//! [handlers.rs]: ./handlers.rs.html
//! [messages.rs]: ./messages.rs.html
//! [flv.rs]: ./flv.rs.html
pub mod handshake;
pub mod messages;
pub mod decoders;
pub mod encoders;
pub mod errors;
pub mod handlers;
pub mod flv;

use std::{
    io::{
        Result as IOResult
    },
    net::{
        TcpListener
    },
    time::{
        SystemTime
    }
};
use self::{
    handlers::*
};

/// # Starts the server.
///
/// This takes following steps respectively:
///
/// 1. Keeps the timestamp
/// 2. Generates the TCP connection
/// 3. Handles the RTMP chunks
///
/// ## Keeps the timestamp
///
/// First, Keeps the timestamp when started.
/// All chunk will refer to this for counting the difference of the timestamp.
///
/// ## Generates the TCP connection
///
/// Generates the TCP connection as "127.0.0.1:1935" (currently).
/// Then, waits for the client to connect to the server.
///
/// ## Handles the RTMP chunks
///
/// Handles the RTMP chunks.
/// Note that in several chunks, sending plural responses will be required, or also sending no response will be required.
/// See handlers.rs for more detail about this phase, and see messages.rs for more detail about each chunk.
///
/// # Errors
///
/// When you got the `ChunkLengthError`:
///
/// * The server couldn't read the chunk completely.
///
/// When you got the `DigestVerificationError`:
///
/// * The HMAC-SHA256 digest didn't find in the C1 chunk.
///
/// When you got the `SignatureDoesNotMatchError`:
///
/// * The HMAC-SHA256 signature in the C2 chunk didn't match with stored one in the server.
///
/// When you got the `ChunkLengthError`:
///
/// * The format of some header or some chunk data is invalid.
///
/// [handlers.rs]: ../handlers.rs.html
/// [messages.rs]: ../messages.rs.html
pub fn run() -> IOResult<()> {
    let start_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let listener = TcpListener::bind("127.0.0.1:1935")?;

    for incoming in listener.incoming() {
        let stream = incoming?;
        let mut handler = RtmpHandler::new(start_time, stream);

        loop {
            handler.handle_chunk()?;
            println!("state: {:?}", handler.get_state());
        }
    }

    Ok(())
}
