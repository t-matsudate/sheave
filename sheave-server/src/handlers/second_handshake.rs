use std::{
    future::Future,
    io::Result as IOResult,
    pin::{
        Pin,
        pin
    },
    task::{
        Context as FutureContext,
        Poll
    }
};
use futures::ready;
use tokio::io::{
    AsyncRead,
    AsyncWrite
};
use sheave_core::{
    handshake::Handshake,
    readers::read_handshake,
    handlers::{
        AsyncHandler,
        RtmpContext,
        inconsistent_sha
    }
};

#[derive(Debug)]
pub struct SecondHandshakeHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for SecondHandshakeHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        let client_response = ready!(pin!(read_handshake(self.0.as_mut())).poll(cx))?;

        if !rtmp_context.is_signed() {
            rtmp_context.set_server_handshake(client_response);
        } else {
            let encryption_algorithm = rtmp_context.get_encryption_algorithm().unwrap();
            let mut client_response_key: Vec<u8> = Vec::new();
            client_response_key.extend_from_slice(Handshake::CLIENT_KEY);
            client_response_key.extend_from_slice(Handshake::COMMON_KEY);
            if !client_response.did_signature_match(encryption_algorithm, &client_response_key) {
                return Poll::Ready(Err(inconsistent_sha(client_response.get_signature().to_vec())))
            } else {
                rtmp_context.set_server_handshake(client_response);
            }
        }

        Poll::Ready(Ok(()))
    }
}

pub fn handle_second_handshake<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> SecondHandshakeHandler<'a, RW> {
    SecondHandshakeHandler(stream)
}

// TODO: cargo check
