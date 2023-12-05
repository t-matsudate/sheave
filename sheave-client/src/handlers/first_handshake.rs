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
        EncryptionAlgorithm,
        Handshake,
        Version
    },
    writers::{
        write_encryption_algorithm,
        write_handshake
    },
    handlers::{
        AsyncHandler,
        RtmpContext
    }
};

#[derive(Debug)]
pub struct FirstHandshakeHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for FirstHandshakeHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        let encryption_algorithm = EncryptionAlgorithm::default();
        let version = if rtmp_context.is_signed() {
            Version::LATEST_CLIENT
        } else {
            Version::UNSIGNED
        };
        let mut client_request = Handshake::new(Instant::now().elapsed(), version);

        if rtmp_context.is_signed() {
            client_request.imprint_digest(encryption_algorithm, Handshake::CLIENT_KEY);
        }

        ready!(pin!(write_encryption_algorithm(self.0.as_mut(), encryption_algorithm)).poll(cx))?;
        ready!(pin!(write_handshake(self.0.as_mut(), &client_request)).poll(cx))?;

        rtmp_context.set_encryption_algorithm(encryption_algorithm);
        rtmp_context.set_client_handshake(client_request);
        Poll::Ready(Ok(()))
    }
}

pub fn handle_first_handshake<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> FirstHandshakeHandler<'a, RW> {
    FirstHandshakeHandler(stream)
}
