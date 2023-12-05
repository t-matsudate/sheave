use std::{
    future::Future,
    io::{
        Error as IOError,
        ErrorKind,
        Result as IOResult
    },
    pin::{
        Pin,
        pin
    },
    task::{
        Context as FutureContext,
        Poll
    },
    time::Instant
};
use futures::ready;
use tokio::io::{
    AsyncRead,
    AsyncWrite
};
use sheave_core::{
    handshake::{
        Handshake,
        Version
    },
    readers::{
        read_encryption_algorithm,
        read_handshake
    },
    writers::{
        write_encryption_algorithm,
        write_handshake
    },
    handlers::{
        AsyncHandler,
        RtmpContext,
        inconsistent_sha
    }
};

#[derive(Debug)]
pub struct FirstHandshakeHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for FirstHandshakeHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        let encryption_algorithm = ready!(pin!(read_encryption_algorithm(self.0.as_mut())).poll(cx))?;
        let mut client_request = ready!(pin!(read_handshake(self.0.as_mut())).poll(cx))?;

        if client_request.get_version() == Version::UNSIGNED {
            let server_request = Handshake::new(Instant::now().elapsed(), Version::UNSIGNED);
            ready!(pin!(write_encryption_algorithm(self.0.as_mut(), encryption_algorithm)).poll(cx))?;
            ready!(pin!(write_handshake(self.0.as_mut(), &server_request)).poll(cx))?;
            ready!(pin!(write_handshake(self.0.as_mut(), &client_request)).poll(cx))?;

            rtmp_context.set_encryption_algorithm(encryption_algorithm);
            rtmp_context.set_server_handshake(server_request);
            rtmp_context.set_client_handshake(client_request);
        } else {
            if !client_request.did_digest_match(encryption_algorithm, Handshake::CLIENT_KEY) {
                return Poll::Ready(
                    Err(
                        IOError::new(
                            ErrorKind::InvalidData,
                            inconsistent_sha(client_request.get_digest(encryption_algorithm).to_vec())
                        )
                    )
                )
            } else {
                let mut server_request = Handshake::new(Instant::now().elapsed(), Version::LATEST_SERVER);
                server_request.imprint_digest(encryption_algorithm, Handshake::SERVER_KEY);
                let mut server_response_key: Vec<u8> = Vec::new();
                server_response_key.extend_from_slice(Handshake::SERVER_KEY);
                server_response_key.extend_from_slice(Handshake::COMMON_KEY);
                client_request.imprint_signature(encryption_algorithm, &server_response_key);
                ready!(pin!(write_encryption_algorithm(self.0.as_mut(), encryption_algorithm)).poll(cx))?;
                ready!(pin!(write_handshake(self.0.as_mut(), &server_request)).poll(cx))?;
                ready!(pin!(write_handshake(self.0.as_mut(), &client_request)).poll(cx))?;

                rtmp_context.set_signed(true);
                rtmp_context.set_encryption_algorithm(encryption_algorithm);
                rtmp_context.set_server_handshake(server_request);
                rtmp_context.set_client_handshake(client_request);
            }
        }

        Poll::Ready(Ok(()))
    }
}

pub fn handle_first_handshake<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> FirstHandshakeHandler<'a, RW> {
    FirstHandshakeHandler(stream)
}
