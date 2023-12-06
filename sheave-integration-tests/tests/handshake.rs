use futures::{
    future::poll_fn
};

#[tokio::test]
async fn first_handshake_unsigned() {
    let result = poll_fn(
        |cx| {
            use std::{
                io::Result as IOResult,
                future::Future,
                pin::pin,
                sync::Arc,
                task::Poll
            };
            use futures::ready;
            use sheave_core::{
                handlers::{
                    AsyncHandler,
                    RtmpContext,
                    StreamWrapper,
                    VecStream
                },
                readers::{
                    read_encryption_algorithm,
                    read_handshake
                }
            };
            use sheave_client::handlers as client;
            use sheave_server::handlers as server;

            let stream = Arc::new(StreamWrapper::new(VecStream::default()));

            let mut client_rtmp_context = RtmpContext::default();
            let result = ready!(pin!(client::handle_first_handshake(stream.make_weak_pin())).poll_handle(cx, &mut client_rtmp_context));
            assert!(result.is_ok());

            let mut server_rtmp_context = RtmpContext::default();
            let result = ready!(pin!(server::handle_first_handshake(stream.make_weak_pin())).poll_handle(cx, &mut server_rtmp_context));
            assert!(result.is_ok());

            let server_encryption_algorithm = ready!(pin!(read_encryption_algorithm(stream.make_weak_pin())).poll(cx)).unwrap();
            // The first step needs just to check whether the client-side request matched with the server.
            ready!(pin!(read_handshake(stream.make_weak_pin())).poll(cx)).unwrap();
            let server_response = ready!(pin!(read_handshake(stream.make_weak_pin())).poll(cx)).unwrap();
            let client_encryption_algorithm = client_rtmp_context.get_encryption_algorithm().unwrap();
            let client_request = client_rtmp_context.get_client_handshake().unwrap();
            assert_eq!(client_encryption_algorithm, server_encryption_algorithm);
            assert_eq!(client_request.get_bytes(), server_response.get_bytes());

            Poll::<IOResult<()>>::Ready(Ok(()))
        }
    ).await;
    assert!(result.is_ok())
}

#[tokio::test]
async fn first_handshake_signed() {
    let result = poll_fn(
        |cx| {
            use std::{
                io::Result as IOResult,
                future::Future,
                pin::pin,
                sync::Arc,
                task::Poll
            };
            use futures::ready;
            use sheave_core::{
                handlers::{
                    AsyncHandler,
                    RtmpContext,
                    StreamWrapper,
                    VecStream
                },
                handshake::Handshake,
                readers::{
                    read_encryption_algorithm,
                    read_handshake
                }
            };
            use sheave_client::handlers as client;
            use sheave_server::handlers as server;

            let stream = Arc::new(StreamWrapper::new(VecStream::default()));

            let mut client_rtmp_context = RtmpContext::default();
            client_rtmp_context.set_signed(true);
            let result = ready!(pin!(client::handle_first_handshake(stream.make_weak_pin())).poll_handle(cx, &mut client_rtmp_context));
            assert!(result.is_ok());

            let mut server_rtmp_context = RtmpContext::default();
            let result = ready!(pin!(server::handle_first_handshake(stream.make_weak_pin())).poll_handle(cx, &mut server_rtmp_context));
            assert!(result.is_ok());

            let server_encryption_algorithm = ready!(pin!(read_encryption_algorithm(stream.make_weak_pin())).poll(cx)).unwrap();
            ready!(pin!(read_handshake(stream.make_weak_pin())).poll(cx)).unwrap();
            let server_response = ready!(pin!(read_handshake(stream.make_weak_pin())).poll(cx)).unwrap();
            let client_encryption_algorithm = client_rtmp_context.get_encryption_algorithm().unwrap();
            let client_request = client_rtmp_context.get_client_handshake_mut().unwrap();
            let mut server_response_key: Vec<u8> = Vec::new();
            server_response_key.extend_from_slice(Handshake::SERVER_KEY);
            server_response_key.extend_from_slice(Handshake::COMMON_KEY);
            client_request.imprint_signature(client_encryption_algorithm, &server_response_key);
            assert_eq!(client_encryption_algorithm, server_encryption_algorithm);
            assert_eq!(client_request.get_bytes(), server_response.get_bytes());

            Poll::<IOResult<()>>::Ready(Ok(()))
        }
    ).await;
    assert!(result.is_ok())
}

#[tokio::test]
async fn second_handshake_unsigned() {
    let result = poll_fn(
        |cx| {
            use std::{
                io::Result as IOResult,
                pin::pin,
                sync::Arc,
                task::Poll
            };
            use futures::ready;
            use sheave_core::{
                handlers::{
                    AsyncHandler,
                    RtmpContext,
                    StreamWrapper,
                    VecStream
                }
            };
            use sheave_client::handlers as client;
            use sheave_server::handlers as server;

            let stream = Arc::new(StreamWrapper::new(VecStream::default()));

            let mut client_rtmp_context = RtmpContext::default();
            let mut server_rtmp_context = RtmpContext::default();
            ready!(pin!(client::handle_first_handshake(stream.make_weak_pin())).poll_handle(cx, &mut client_rtmp_context)).unwrap();
            ready!(pin!(server::handle_first_handshake(stream.make_weak_pin())).poll_handle(cx, &mut server_rtmp_context)).unwrap();

            let result = ready!(pin!(client::handle_second_handshake(stream.make_weak_pin())).poll_handle(cx, &mut client_rtmp_context));
            assert!(result.is_ok());

            let result = ready!(pin!(server::handle_second_handshake(stream.make_weak_pin())).poll_handle(cx, &mut server_rtmp_context));
            assert!(result.is_ok());

            let client_encryption_algorithm = client_rtmp_context.get_encryption_algorithm().unwrap();
            let server_encryption_algorithm = server_rtmp_context.get_encryption_algorithm().unwrap();
            assert_eq!(client_encryption_algorithm, server_encryption_algorithm);

            let client_request = client_rtmp_context.get_client_handshake().unwrap();
            let server_response = server_rtmp_context.get_client_handshake().unwrap();
            assert_eq!(client_request.get_bytes(), server_response.get_bytes());

            let server_request = server_rtmp_context.get_server_handshake().unwrap();
            let client_response = client_rtmp_context.get_server_handshake().unwrap();
            assert_eq!(server_request.get_bytes(), client_response.get_bytes());

            Poll::<IOResult<()>>::Ready(Ok(()))
        }
    ).await;
    assert!(result.is_ok())
}

#[tokio::test]
async fn second_handshake_signed() {
    let result = poll_fn(
        |cx| {
            use std::{
                io::Result as IOResult,
                pin::pin,
                sync::Arc,
                task::Poll
            };
            use futures::ready;
            use sheave_core::{
                handlers::{
                    AsyncHandler,
                    RtmpContext,
                    StreamWrapper,
                    VecStream
                }
            };
            use sheave_client::handlers as client;
            use sheave_server::handlers as server;

            let stream = Arc::new(StreamWrapper::new(VecStream::default()));
            let mut client_rtmp_context = RtmpContext::default();
            let mut server_rtmp_context = RtmpContext::default();
            client_rtmp_context.set_signed(true);

            ready!(pin!(client::handle_first_handshake(stream.make_weak_pin())).poll_handle(cx, &mut client_rtmp_context)).unwrap();
            ready!(pin!(server::handle_first_handshake(stream.make_weak_pin())).poll_handle(cx, &mut server_rtmp_context)).unwrap();

            let result = ready!(pin!(client::handle_second_handshake(stream.make_weak_pin())).poll_handle(cx, &mut client_rtmp_context));
            assert!(result.is_ok());

            let result = ready!(pin!(server::handle_second_handshake(stream.make_weak_pin())).poll_handle(cx, &mut server_rtmp_context));
            assert!(result.is_ok());

            let client_encryption_algorithm = client_rtmp_context.get_encryption_algorithm().unwrap();
            let server_encryption_algorithm = server_rtmp_context.get_encryption_algorithm().unwrap();
            assert_eq!(client_encryption_algorithm, server_encryption_algorithm);

            let client_request = client_rtmp_context.get_client_handshake().unwrap();
            let server_response = server_rtmp_context.get_client_handshake().unwrap();
            assert_eq!(client_request.get_bytes(), server_response.get_bytes());

            let server_request = server_rtmp_context.get_server_handshake().unwrap();
            let client_response = client_rtmp_context.get_server_handshake().unwrap();
            assert_eq!(server_request.get_bytes(), client_response.get_bytes());

            Poll::<IOResult<()>>::Ready(Ok(()))
        }
    ).await;
    assert!(result.is_ok())
}
