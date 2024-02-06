use futures::future::poll_fn;

#[tokio::test]
async fn information_exchange_with_default() {
    let result = poll_fn(
        |cx| {
            use std::{
                io::Result as IOResult,
                future::Future,
                pin::pin,
                sync::Arc,
                task::Poll,
                time::Duration
            };
            use futures::ready;
            use sheave_core::{
                ByteBuffer,
                Encoder,
                handlers::{
                    AsyncHandler,
                    RtmpContext,
                    StreamWrapper,
                    VecStream
                },
                messages::{
                    ChunkData,
                    ConnectResult,
                    ReleaseStreamResult,
                    OnFcPublish,
                    CreateStream,
                    CreateStreamResult,
                    amf::v0::{
                        Number,
                        AmfString
                    },
                    headers::{
                        BasicHeader,
                        MessageFormat,
                        MessageHeader
                    }
                },
                object,
                readers::read_chunk,
                writers::{
                    write_basic_header,
                    write_message_header,
                    write_chunk_data
                }
            };
            use sheave_client::handlers as client;
            use sheave_server::handlers as server;

            let stream = Arc::new(StreamWrapper::new(VecStream::default()));
            let mut client_rtmp_context = RtmpContext::default();
            let mut server_rtmp_context = RtmpContext::default();

            // Hnadling "connect".
            // NOTE: Because currently client-side handler receives a result message simultaneously with sending its request.
            let expected_connect_result = ConnectResult::new(
                "_result".into(),
                object!(
                    "fmsVer" => AmfString::from("FMS/5,0,17"),
                    "capabilities" => Number::from(31)
                ),
                object!(
                    "level" => AmfString::from("status"),
                    "code" => AmfString::from("NetConnection.Connect.Success"),
                    "description" => AmfString::from("Connection succeeded."),
                    "objectEncoding" => Number::from(0)
                )
            );
            let mut buffer = ByteBuffer::default();
            buffer.encode(&expected_connect_result);
            let data: Vec<u8> = buffer.into();
            ready!(pin!(write_basic_header(stream.make_weak_pin(), &BasicHeader::new(MessageFormat::New, ConnectResult::CHANNEL as u16))).poll(cx))?;
            ready!(pin!(write_message_header(stream.make_weak_pin(), &MessageHeader::New((Duration::default(), data.len() as u32, ConnectResult::MESSAGE_TYPE, u32::default()).into()))).poll(cx))?;
            ready!(pin!(write_chunk_data(stream.make_weak_pin(), ConnectResult::CHANNEL as u16, client_rtmp_context.get_sending_chunk_size(), &data)).poll(cx))?;
            let result = ready!(pin!(client::handle_connect(stream.make_weak_pin())).poll_handle(cx, &mut client_rtmp_context));
            assert!(result.is_ok());
            let result = ready!(pin!(server::handle_connect(stream.make_weak_pin())).poll_handle(cx, &mut server_rtmp_context));
            assert!(result.is_ok());
            let actual_connect_result: ConnectResult = ready!(pin!(read_chunk(stream.make_weak_pin(), &mut client_rtmp_context)).poll(cx))?;
            assert_eq!(expected_connect_result, actual_connect_result);

            // Handling "releaseStream".
            let expected_release_stream_result = ReleaseStreamResult::new("_result".into(), 2u8.into());
            let mut buffer = ByteBuffer::default();
            buffer.encode(&expected_release_stream_result);
            let data: Vec<u8> = buffer.into();
            ready!(pin!(write_basic_header(stream.make_weak_pin(), &BasicHeader::new(MessageFormat::SameSource, ReleaseStreamResult::CHANNEL as u16))).poll(cx))?;
            ready!(pin!(write_message_header(stream.make_weak_pin(), &MessageHeader::SameSource((Duration::default(), data.len() as u32, ReleaseStreamResult::MESSAGE_TYPE).into()))).poll(cx))?;
            ready!(pin!(write_chunk_data(stream.make_weak_pin(), ReleaseStreamResult::CHANNEL as u16, client_rtmp_context.get_sending_chunk_size(), &data)).poll(cx))?;
            let result = ready!(pin!(client::handle_release_stream(stream.make_weak_pin())).poll_handle(cx, &mut client_rtmp_context));
            assert!(result.is_ok());
            let result = ready!(pin!(server::handle_release_stream(stream.make_weak_pin())).poll_handle(cx, &mut server_rtmp_context));
            assert!(result.is_ok());
            let actual_release_stream_result: ReleaseStreamResult = ready!(pin!(read_chunk(stream.make_weak_pin(), &mut client_rtmp_context)).poll(cx))?;
            assert_eq!(expected_release_stream_result, actual_release_stream_result);

            // Handling "FCPublish".
            let expected_on_fc_publish = OnFcPublish;
            let mut buffer = ByteBuffer::default();
            buffer.encode(&expected_on_fc_publish);
            let data: Vec<u8> = buffer.into();
            ready!(pin!(write_basic_header(stream.make_weak_pin(), &BasicHeader::new(MessageFormat::SameSource, OnFcPublish::CHANNEL as u16))).poll(cx))?;
            ready!(pin!(write_message_header(stream.make_weak_pin(), &MessageHeader::SameSource((Duration::default(), data.len() as u32, OnFcPublish::MESSAGE_TYPE).into()))).poll(cx))?;
            ready!(pin!(write_chunk_data(stream.make_weak_pin(), OnFcPublish::CHANNEL as u16, client_rtmp_context.get_sending_chunk_size(), &data)).poll(cx))?;
            let result = ready!(pin!(client::handle_fc_publish(stream.make_weak_pin())).poll_handle(cx, &mut client_rtmp_context));
            assert!(result.is_ok());
            let result = ready!(pin!(server::handle_fc_publish(stream.make_weak_pin())).poll_handle(cx, &mut server_rtmp_context));
            assert!(result.is_ok());
            let actual_on_fc_publish: OnFcPublish = ready!(pin!(read_chunk(stream.make_weak_pin(), &mut client_rtmp_context)).poll(cx))?;
            assert_eq!(expected_on_fc_publish, actual_on_fc_publish);

            let expected_create_stream_result = CreateStreamResult::new("_result".into(), 4u8.into(), Number::default());
            let mut buffer = ByteBuffer::default();
            buffer.encode(&expected_create_stream_result);
            let data: Vec<u8> = buffer.into();
            ready!(pin!(write_basic_header(stream.make_weak_pin(), &BasicHeader::new(MessageFormat::SameSource, CreateStreamResult::CHANNEL as u16))).poll(cx))?;
            ready!(pin!(write_message_header(stream.make_weak_pin(), &MessageHeader::SameSource((Duration::default(), data.len() as u32, CreateStreamResult::MESSAGE_TYPE).into()))).poll(cx))?;
            ready!(pin!(write_chunk_data(stream.make_weak_pin(), CreateStreamResult::CHANNEL as u16, client_rtmp_context.get_sending_chunk_size(), &data)).poll(cx))?;
            let result = ready!(pin!(client::handle_create_stream(stream.make_weak_pin())).poll_handle(cx, &mut client_rtmp_context));
            assert!(result.is_ok());
            let result = ready!(pin!(server::handle_create_stream(stream.make_weak_pin())).poll_handle(cx, &mut server_rtmp_context));
            assert!(result.is_ok());
            let actual_create_stream_result: CreateStreamResult = ready!(pin!(read_chunk(stream.make_weak_pin(), &mut client_rtmp_context)).poll(cx))?;
            assert_eq!(expected_create_stream_result, actual_create_stream_result);

            Poll::<IOResult<()>>::Ready(Ok(()))
        }
    ).await;
    assert!(result.is_ok())
}
