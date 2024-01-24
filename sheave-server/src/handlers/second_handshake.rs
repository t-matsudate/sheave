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

#[doc(hidden)]
#[derive(Debug)]
pub struct SecondHandshakeHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
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
            let server_request = rtmp_context.get_server_handshake().unwrap();
            // Note the FFmpeg acts the handshake but imprints no signature.
            if !client_response.did_signature_match(encryption_algorithm, &client_response_key) && server_request.get_signature() != client_response.get_signature() {
                return Poll::Ready(Err(inconsistent_sha(client_response.get_signature().to_vec())))
            } else {
                rtmp_context.set_server_handshake(client_response);
            }
        }

        Poll::Ready(Ok(()))
    }
}

/// Handles a handshake chunk of the second step as a server.
/// This step performs:
///
/// 1. Receives a handshake chunk which is a response from a client.
/// 2. Validates a signature imprinted into its response. If it is valid, servers go to next step. Otherwise terminate its connection as an error.
///
/// Note:
///
/// * Also if version bytes are specified 0, (that is, it has no signature,) servers go to next step as it is. Because of nothing to validate.
/// * Somehow some client exists not imprinting a signature though imprints a digest. e.g. FFmpeg
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
///                 },
///                 writers::{
///                     write_encryption_algorithm,
///                     write_handshake
///                 }
///             };
///             use sheave_server::handlers::handle_second_handshake;
///
///             // When without any signature.
///             let mut stream = pin!(VecStream::default());
///             let mut rtmp_context = RtmpContext::default();
///             let response = Handshake::new(Duration::default(), Version::UNSIGNED);
///             rtmp_context.set_encryption_algorithm(EncryptionAlgorithm::NotEncrypted);
///             rtmp_context.set_server_handshake(response);
///             ready!(pin!(write_handshake(stream.as_mut(), rtmp_context.get_server_handshake().unwrap())).poll(cx))?;
///             let result = ready!(pin!(handle_second_handshake(stream.as_mut())).poll_handle(cx, &mut rtmp_context));
///             assert!(result.is_ok());
///
///             // When with some signature.
///             let mut stream = pin!(VecStream::default());
///             let mut rtmp_context = RtmpContext::default();
///             let mut response = Handshake::new(Duration::default(), Version::LATEST_SERVER);
///             rtmp_context.set_signed(true);
///             rtmp_context.set_encryption_algorithm(EncryptionAlgorithm::NotEncrypted);
///             let mut client_response_key: Vec<u8> = Vec::new();
///             client_response_key.extend_from_slice(Handshake::CLIENT_KEY);
///             client_response_key.extend_from_slice(Handshake::COMMON_KEY);
///             response.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY);
///             response.imprint_signature(EncryptionAlgorithm::NotEncrypted, &client_response_key);
///             rtmp_context.set_server_handshake(response);
///             ready!(pin!(write_handshake(stream.as_mut(), rtmp_context.get_server_handshake().unwrap())).poll(cx))?;
///             let result = ready!(pin!(handle_second_handshake(stream.as_mut())).poll_handle(cx, &mut rtmp_context));
///             assert!(result.is_ok());
///
///             // In the special case (FFmpeg)
///             let mut stream = pin!(VecStream::default());
///             let mut rtmp_context = RtmpContext::default();
///             let mut response = Handshake::new(Duration::default(), Version::LATEST_SERVER);
///             rtmp_context.set_signed(true);
///             rtmp_context.set_encryption_algorithm(EncryptionAlgorithm::NotEncrypted);
///             response.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY);
///             rtmp_context.set_server_handshake(response);
///             ready!(pin!(write_handshake(stream.as_mut(), rtmp_context.get_server_handshake().unwrap())).poll(cx))?;
///             let result = ready!(pin!(handle_second_handshake(stream.as_mut())).poll_handle(cx, &mut rtmp_context));
///             // Somehow currently the FFmpeg don't imprint its signature.
///             assert!(result.is_ok());
///
///             Poll::Ready(Ok(()))
///         }
///     ).await;
///     assert!(result.is_ok());
///
///     Ok(())
/// }
/// ```
pub fn handle_second_handshake<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> SecondHandshakeHandler<'a, RW> {
    SecondHandshakeHandler(stream)
}
