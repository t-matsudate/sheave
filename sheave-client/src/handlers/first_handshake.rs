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

#[doc(hidden)]
#[derive(Debug)]
pub struct FirstHandshakeHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
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

/// Handles a handshake chunk of the first step as a client.
///
/// # Examples
///
/// ```rust
/// use std::io::Result as IOResult;
/// use futures::future::poll_fn;
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let result: IOResult<()> = poll_fn(
///         |cx| {
///             use std::{
///                 future::Future,
///                 pin::pin,
///                 task::Poll,
///                 time::Duration
///             };
///             use futures::ready;
///             use sheave_core::{
///                 handlers::{
///                     AsyncHandler,
///                     RtmpContext,
///                     VecStream
///                 },
///                 handshake::{
///                     EncryptionAlgorithm,
///                     Handshake,
///                     Version
///                 },
///                 readers::{
///                     read_encryption_algorithm,
///                     read_handshake
///                 }
///             };
///             use sheave_client::handlers::handle_first_handshake;
///
///             // When without any digest.
///             let mut stream = pin!(VecStream::default());
///             let mut rtmp_context = RtmpContext::default();
///
///             ready!(pin!(handle_first_handshake(stream.as_mut())).poll_handle(cx, &mut rtmp_context))?;
///
///             let encryption_algorithm = ready!(pin!(read_encryption_algorithm(stream.as_mut())).poll(cx))?;
///             assert_eq!(EncryptionAlgorithm::NotEncrypted, encryption_algorithm);
///             let handshake = ready!(pin!(read_handshake(stream.as_mut())).poll(cx))?;
///             assert_eq!(Version::UNSIGNED, handshake.get_version());
///
///             // When with some digest.
///             let mut stream = pin!(VecStream::default());
///             let mut rtmp_context = RtmpContext::default();
///             rtmp_context.set_signed(true);
///
///             ready!(pin!(handle_first_handshake(stream.as_mut())).poll_handle(cx, &mut rtmp_context))?;
///
///             let encryption_algorithm = ready!(pin!(read_encryption_algorithm(stream.as_mut())).poll(cx))?;
///             assert_eq!(EncryptionAlgorithm::NotEncrypted, encryption_algorithm);
///             let handshake = ready!(pin!(read_handshake(stream.as_mut())).poll(cx))?;
///             assert_eq!(Version::LATEST_CLIENT, handshake.get_version());
///             assert!(handshake.did_digest_match(encryption_algorithm, Handshake::CLIENT_KEY));
///
///             Poll::Ready(Ok(()))
///         }
///     ).await;
///     assert!(result.is_ok());
///
///     Ok(())
/// }
/// ```
pub fn handle_first_handshake<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> FirstHandshakeHandler<'a, RW> {
    FirstHandshakeHandler(stream)
}
