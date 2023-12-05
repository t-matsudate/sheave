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
    readers::{
        read_encryption_algorithm,
        read_handshake
    },
    writers::{
        write_handshake
    },
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
        let encryption_algorithm = ready!(pin!(read_encryption_algorithm(self.0.as_mut())).poll(cx))?;
        let mut server_request = ready!(pin!(read_handshake(self.0.as_mut())).poll(cx))?;
        let server_response = ready!(pin!(read_handshake(self.0.as_mut())).poll(cx))?;

        if !rtmp_context.is_signed() {
            ready!(pin!(write_handshake(self.0.as_mut(), &server_request)).poll(cx))?;

            rtmp_context.set_server_handshake(server_request);
            rtmp_context.set_client_handshake(server_response);
        } else if !server_request.did_digest_match(encryption_algorithm, Handshake::SERVER_KEY) {
            return Poll::Ready(Err(inconsistent_sha(server_request.get_digest(encryption_algorithm).to_vec())))
        } else {
            let mut server_response_key: Vec<u8> = Vec::new();
            server_response_key.extend_from_slice(Handshake::SERVER_KEY);
            server_response_key.extend_from_slice(Handshake::COMMON_KEY);

            if !server_response.did_signature_match(encryption_algorithm, &server_response_key) {
                return Poll::Ready(Err(inconsistent_sha(server_response.get_signature().to_vec())))
            } else {
                let mut client_response_key: Vec<u8> = Vec::new();
                client_response_key.extend_from_slice(Handshake::CLIENT_KEY);
                client_response_key.extend_from_slice(Handshake::COMMON_KEY);
                server_request.imprint_signature(encryption_algorithm, &client_response_key);
                ready!(pin!(write_handshake(self.0.as_mut(), &server_request)).poll(cx))?;

                rtmp_context.set_signed(true);
                rtmp_context.set_server_handshake(server_request);
                rtmp_context.set_client_handshake(server_response);
            }
        }

        Poll::Ready(Ok(()))
    }
}

pub fn handle_second_handshake<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> SecondHandshakeHandler<'a, RW> {
    SecondHandshakeHandler(stream)
}
