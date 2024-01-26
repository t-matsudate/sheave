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

#[doc(hidden)]
#[derive(Debug)]
pub struct FirstHandshakeHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
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

/// Handles a handshake chunk of the first step as a server.
/// This step performs:
///
/// 1. Receives a handshake chunk from a client.
/// 2. If it is imprinted some digest, validates it.
/// 3. Makes a response chunk from a client's request. If it is imprinted some digest, also we are required to imprint our signature into it.
/// 4. Sends it with a server's request to a client..
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
///             use sheave_server::handlers::handle_first_handshake;
///
///             // When without any digest.
///             let mut stream = pin!(VecStream::default());
///             let expected_encryption_algorithm = EncryptionAlgorithm::NotEncrypted;
///             let expected_handshake = Handshake::new(Duration::default(), Version::UNSIGNED);
///             ready!(pin!(write_encryption_algorithm(stream.as_mut(), expected_encryption_algorithm)).poll(cx))?;
///             ready!(pin!(write_handshake(stream.as_mut(), &expected_handshake)).poll(cx))?;
///
///             ready!(pin!(handle_first_handshake(stream.as_mut())).poll_handle(cx, &mut RtmpContext::default()))?;
///
///             let actual_encryption_algorithm = ready!(pin!(read_encryption_algorithm(stream.as_mut())).poll(cx))?;
///             assert_eq!(expected_encryption_algorithm, actual_encryption_algorithm);
///             // In this case, server's handshake isn't required to verify because is without any digest.
///             ready!(pin!(read_handshake(stream.as_mut())).poll(cx))?;
///             let actual_handshake = ready!(pin!(read_handshake(stream.as_mut())).poll(cx))?;
///             assert_eq!(expected_handshake.get_bytes(), actual_handshake.get_bytes());
///
///             // When with some digest/signature.
///             let mut stream = pin!(VecStream::default());
///             let expected_encryption_algorithm = EncryptionAlgorithm::NotEncrypted;
///             let mut expected_handshake = Handshake::new(Duration::default(), Version::LATEST_CLIENT);
///             expected_handshake.imprint_digest(expected_encryption_algorithm, Handshake::CLIENT_KEY);
///             ready!(pin!(write_encryption_algorithm(stream.as_mut(), expected_encryption_algorithm)).poll(cx))?;
///             ready!(pin!(write_handshake(stream.as_mut(), &expected_handshake)).poll(cx))?;
///
///             ready!(pin!(handle_first_handshake(stream.as_mut())).poll_handle(cx, &mut RtmpContext::default()))?;
///
///             let actual_encryption_algorithm = ready!(pin!(read_encryption_algorithm(stream.as_mut())).poll(cx))?;
///             assert_eq!(expected_encryption_algorithm, actual_encryption_algorithm);
///             let server_handshake = ready!(pin!(read_handshake(stream.as_mut())).poll(cx))?;
///             // If some digest is imprinted, it matches with server's one.
///             assert!(server_handshake.did_digest_match(actual_encryption_algorithm, Handshake::SERVER_KEY));
///             let actual_handshake = ready!(pin!(read_handshake(stream.as_mut())).poll(cx))?;
///             let mut server_response_key: Vec<u8> = Vec::new();
///             server_response_key.extend_from_slice(Handshake::SERVER_KEY);
///             server_response_key.extend_from_slice(Handshake::COMMON_KEY);
///             // Also a signature matches with server's one.
///             assert!(actual_handshake.did_signature_match(actual_encryption_algorithm, &server_response_key));
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
