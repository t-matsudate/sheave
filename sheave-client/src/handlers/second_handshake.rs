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

#[doc(hidden)]
#[derive(Debug)]
pub struct SecondHandshakeHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
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

/// Handles a handshake chunk of the second step as a client.
/// This step performs:
///
/// 1. Receives server's handshake chunks both a request and a response.
/// 2. If it is imprinted some digest/signature, validates it.
/// 3. Makes a response chunk from a server's request. If it is imprinted some digest, also we are reruired to imprint our signature into it.
/// 4. Sends it to a server.
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
///                 readers::read_handshake,
///                 writers::{
///                     write_encryption_algorithm,
///                     write_handshake
///                 }
///             };
///             use sheave_client::handlers::handle_second_handshake;
///
///             // When without any digest.
///             let mut stream = pin!(VecStream::default());
///             let mut rtmp_context = RtmpContext::default();
///
///             ready!(pin!(write_encryption_algorithm(stream.as_mut(), EncryptionAlgorithm::NotEncrypted)).poll(cx))?;
///             ready!(pin!(write_handshake(stream.as_mut(), &Handshake::new(Duration::default(), Version::UNSIGNED))).poll(cx))?;
///             ready!(pin!(write_handshake(stream.as_mut(), &Handshake::new(Duration::default(), Version::UNSIGNED))).poll(cx))?;
///
///             ready!(pin!(handle_second_handshake(stream.as_mut())).poll_handle(cx, &mut rtmp_context))?;
///
///             let handshake = ready!(pin!(read_handshake(stream.as_mut())).poll(cx))?;
///             assert_eq!(Version::UNSIGNED, handshake.get_version());
///
///             // When with some digest/signature.
///             let mut stream = pin!(VecStream::default());
///             let mut rtmp_context = RtmpContext::default();
///             rtmp_context.set_signed(true);
///             let mut server_request = Handshake::new(Duration::default(), Version::LATEST_SERVER);
///             server_request.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY);
///             let mut server_response = Handshake::new(Duration::default(), Version::LATEST_CLIENT);
///             server_response.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY);
///             let mut server_response_key: Vec<u8> = Vec::new();
///             server_response_key.extend_from_slice(Handshake::SERVER_KEY);
///             server_response_key.extend_from_slice(Handshake::COMMON_KEY);
///             server_response.imprint_signature(EncryptionAlgorithm::NotEncrypted, &server_response_key);
///
///             ready!(pin!(write_encryption_algorithm(stream.as_mut(), EncryptionAlgorithm::NotEncrypted)).poll(cx))?;
///             ready!(pin!(write_handshake(stream.as_mut(), &server_request)).poll(cx))?;
///             ready!(pin!(write_handshake(stream.as_mut(), &server_response)).poll(cx))?;
///
///             ready!(pin!(handle_second_handshake(stream.as_mut())).poll_handle(cx, &mut rtmp_context))?;
///
///             let client_response = ready!(pin!(read_handshake(stream.as_mut())).poll(cx))?;
///             let mut client_response_key: Vec<u8> = Vec::new();
///             client_response_key.extend_from_slice(Handshake::CLIENT_KEY);
///             client_response_key.extend_from_slice(Handshake::COMMON_KEY);
///             assert_eq!(Version::LATEST_SERVER, client_response.get_version());
///             assert!(client_response.did_signature_match(EncryptionAlgorithm::NotEncrypted, &client_response_key));
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
