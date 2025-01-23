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
    sync::Arc,
    task::{
        Context as FutureContext,
        Poll
    },
    time::{
        Duration,
        Instant
    }
};
use log::{
    debug,
    error,
    info
};
use futures::ready;
use tokio::{
    io::{
        AsyncRead,
        AsyncWrite
    }
};
use uuid::Uuid;
use sheave_core::{
    ByteBuffer,
    Decoder,
    Encoder,
    U24_MAX,
    flv::{
        Flv,
        tags::*
    },
    handlers::{
        AsyncHandler,
        AsyncHandlerExt,
        ErrorHandler,
        HandlerConstructor,
        LastChunk,
        PublisherStatus,
        RtmpContext,
        StreamWrapper,
        inconsistent_sha
    },
    handshake::{
        Handshake,
        Version
    },
    messages::{
        Acknowledgement,
        ChunkData,
        Connect,
        ConnectResult,
        CreateStream,
        CreateStreamResult,
        DeleteStream,
        FcPublish,
        FcUnpublish,
        OnFcPublish,
        OnStatus,
        Publish,
        ReleaseStream,
        ReleaseStreamResult,
        StreamBegin,
        UserControl,
        amf::v0::{
            AmfString,
            Number
        },
        headers::MessageType
    },
    net::RtmpReadExt,
    object,
    readers::*,
    writers::*
};
use crate::server::{
    provide_message_id,
    return_message_id
};
use super::middlewares::write_acknowledgement;

#[doc(hidden)]
#[derive(Debug)]
struct HandshakeHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> HandshakeHandler<'_, RW> {
    async fn handle_first_handshake(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let encryption_algorithm = read_encryption_algorithm(pin!(self.0.await_until_receiving())).await?;
        let mut client_request = read_handshake(pin!(self.0.await_until_receiving())).await?;

        if client_request.get_version() == Version::UNSIGNED {
            let server_request = Handshake::new(Instant::now().elapsed(), Version::UNSIGNED);
            write_encryption_algorithm(self.0.as_mut(), encryption_algorithm).await?;
            write_handshake(self.0.as_mut(), &server_request).await?;
            write_handshake(self.0.as_mut(), &client_request).await?;

            rtmp_context.set_encryption_algorithm(encryption_algorithm);
            rtmp_context.set_server_handshake(server_request);
            rtmp_context.set_client_handshake(client_request);
        } else {
            if !client_request.did_digest_match(encryption_algorithm, Handshake::CLIENT_KEY) {
                error!("Client side digest is inconsistent!\nencryption_algorithm: {:?}\ndigest: {:x?}", encryption_algorithm, client_request.get_digest(encryption_algorithm));
                return Err(
                    IOError::new(
                        ErrorKind::InvalidData,
                        inconsistent_sha(client_request.get_digest(encryption_algorithm).to_vec())
                    )
                )
            } else {
                let mut server_request = Handshake::new(Instant::now().elapsed(), Version::LATEST_SERVER);
                server_request.imprint_digest(encryption_algorithm, Handshake::SERVER_KEY);
                let mut server_response_key: Vec<u8> = Vec::new();
                server_response_key.extend_from_slice(Handshake::SERVER_KEY);
                server_response_key.extend_from_slice(Handshake::COMMON_KEY);
                client_request.imprint_signature(encryption_algorithm, &server_response_key);
                write_encryption_algorithm(self.0.as_mut(), encryption_algorithm).await?;
                write_handshake(self.0.as_mut(), &server_request).await?;
                write_handshake(self.0.as_mut(), &client_request).await?;

                rtmp_context.set_signed(true);
                rtmp_context.set_encryption_algorithm(encryption_algorithm);
                rtmp_context.set_server_handshake(server_request);
                rtmp_context.set_client_handshake(client_request);
            }
        }

        info!("First handshake got handled.");
        Ok(())
    }

    async fn handle_second_handshake(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let client_response = read_handshake(pin!(self.0.await_until_receiving())).await?;

        if !rtmp_context.is_signed() {
            rtmp_context.set_server_handshake(client_response);
        } else {
            let encryption_algorithm = rtmp_context.get_encryption_algorithm().unwrap();
            let mut client_response_key: Vec<u8> = Vec::new();
            client_response_key.extend_from_slice(Handshake::CLIENT_KEY);
            client_response_key.extend_from_slice(Handshake::COMMON_KEY);
            let server_request = rtmp_context.get_server_handshake().unwrap();
            // NOTE: FFmpeg acts the handshake but imprints no signature.
            if !client_response.did_signature_match(encryption_algorithm, &client_response_key) && server_request.get_signature() != client_response.get_signature() {
                error!("Client side signature is inconsistent!\nencryption_algorithm: {:?}, signature: {:x?}", encryption_algorithm, client_response.get_signature());
                return Err(inconsistent_sha(client_response.get_signature().to_vec()))
            } else {
                debug!("Handshake version: {:?}\nSignature: {:x?}", client_response.get_version(), client_response.get_signature());
                rtmp_context.set_server_handshake(client_response);
            }
        }

        info!("Second handshake got handled.");
        Ok(())
    }
}

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for HandshakeHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        ready!(pin!(self.handle_first_handshake(rtmp_context)).poll(cx))?;
        pin!(self.handle_second_handshake(rtmp_context)).poll(cx)
    }
}

#[doc(hidden)]
fn handle_handshake<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> HandshakeHandler<'a, RW> {
    HandshakeHandler(stream)
}

#[doc(hidden)]
#[derive(Debug)]
struct MessageHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> MessageHandler<'_, RW> {
    async fn handle_acknowledgement(&mut self, _: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<Acknowledgement>::decode(&mut buffer)?;

        info!("Acknowledgement got handled.");
        Ok(())
    }

    async fn handle_connect_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let connect_request: Connect = buffer.decode()?;
        rtmp_context.set_command_object(connect_request.into());

        info!("connect got handled.");
        Ok(())
    }

    async fn handle_release_stream_request(&mut self, _: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<ReleaseStream>::decode(&mut buffer)?;

        info!("releaseStream got handled.");
        Ok(())
    }

    async fn handle_fc_publish_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let fc_publish_request: FcPublish = buffer.decode()?;
        let mut playpath: AmfString = fc_publish_request.into();

        if playpath.is_empty() {
            playpath = AmfString::new(format!("/tmp/{}.flv", Uuid::new_v4()));
        }

        let input = Flv::create(&playpath)?;
        rtmp_context.set_input(input);
        rtmp_context.set_playpath(playpath);

        info!("FCPublish got handled.");
        Ok(())
    }

    async fn handle_create_stream_request(&mut self, _: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<CreateStream>::decode(&mut buffer)?;

        info!("createStream got handled.");
        Ok(())
    }

    async fn handle_publish_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let publish_request: Publish = buffer.decode()?;
        let (publishing_name, publishing_type): (AmfString, AmfString) = publish_request.into();
        rtmp_context.set_publishing_name(publishing_name);
        rtmp_context.set_publishing_type(publishing_type);

        info!("publish got handled.");
        Ok(())
    }

    async fn handle_fc_unpublish_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<FcUnpublish>::decode(&mut buffer)?;

        rtmp_context.reset_playpath();

        info!("FCUnpublish got handled.");
        Ok(())
    }

    async fn handle_delete_stream_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let delete_stream_request: DeleteStream = buffer.decode()?;
        return_message_id(delete_stream_request.into());
        rtmp_context.reset_message_id();

        info!("deleteStream got handled.");
        Ok(())
    }

    async fn handle_command_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use PublisherStatus::*;

        let command: AmfString = buffer.decode()?;
        let transaction_id: Number = buffer.decode()?;
        rtmp_context.set_transaction_id(transaction_id);

        if command == "FCUnpublish" {
            return self.handle_fc_unpublish_request(rtmp_context, buffer).await
        } else if command == "deleteStream" {
            return self.handle_delete_stream_request(rtmp_context, buffer).await
        } else {
            /* In this step, does nothing unless command is either "FCUnpublish" or "deleteStream". */
        }

        if let Some(publisher_status) = rtmp_context.get_publisher_status() {
            match publisher_status {
                Connected => self.handle_release_stream_request(rtmp_context, buffer).await,
                Released => self.handle_fc_publish_request(rtmp_context, buffer).await,
                FcPublished => self.handle_create_stream_request(rtmp_context, buffer).await,
                Created => self.handle_publish_request(rtmp_context, buffer).await,
                _ => Ok(())
            }
        } else {
            self.handle_connect_request(rtmp_context, buffer).await
        }
    }

    async fn handle_flv(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer, message_type: MessageType, timestamp: Duration) -> IOResult<()> {
        let input = rtmp_context.get_input().unwrap();

        let tag_type = match message_type {
            MessageType::Audio => TagType::Audio,
            MessageType::Video => TagType::Video,
            MessageType::Data => TagType::ScriptData,
            _ => TagType::Other
        };

        if let TagType::ScriptData = tag_type {
            // NOTE: Currently @setDataFrame command is used for nothing.
            Decoder::<AmfString>::decode(&mut buffer)?;
        }

        let data: Vec<u8> = buffer.into();
        let flv_tag = FlvTag::new(tag_type, timestamp, data);
        input.append_flv_tag(flv_tag)?;

        info!("FLV chunk got handled.");
        Ok(())
    }

    async fn write_connect_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let properties = object!(
            "fmsVer" => AmfString::from("FMS/5,0,17"),
            "capabilities" => Number::from(31)
        );
        let information = object!(
            "level" => AmfString::from("status"),
            "code" => AmfString::from("NetConnection.Connect.Success"),
            "description" => AmfString::from("Connection succeeded."),
            "objectEncoding" => Number::from(0)
        );
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("_result"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&ConnectResult::new(properties.clone(), information.clone()));
        write_chunk(self.0.as_mut(), rtmp_context, ConnectResult::CHANNEL.into(), Duration::default(), ConnectResult::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;
        rtmp_context.set_properties(properties);
        rtmp_context.set_information(information);

        rtmp_context.set_publisher_status(PublisherStatus::Connected);

        info!("connect result got sent.");
        Ok(())
    }

    async fn write_release_stream_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("_result"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&ReleaseStreamResult);
        write_chunk(self.0.as_mut(), rtmp_context, ReleaseStreamResult::CHANNEL.into(), Duration::default(), ReleaseStreamResult::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_publisher_status(PublisherStatus::Released);

        info!("releaseStream result got sent.");
        Ok(())
    }

    async fn write_fc_publish_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("onFCPublish"));
        buffer.encode(&OnFcPublish);
        write_chunk(self.0.as_mut(), rtmp_context, OnFcPublish::CHANNEL.into(), Duration::default(), OnFcPublish::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_publisher_status(PublisherStatus::FcPublished);

        info!("onFCPublish got sent.");
        Ok(())
    }

    async fn write_create_stream_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let message_id = provide_message_id();
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("_result"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&CreateStreamResult::new(message_id.into()));
        write_chunk(self.0.as_mut(), rtmp_context, CreateStreamResult::CHANNEL.into(), Duration::default(), CreateStreamResult::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;
        rtmp_context.set_message_id(message_id);

        rtmp_context.set_publisher_status(PublisherStatus::Created);

        info!("createStream result got sent.");
        Ok(())
    }

    async fn write_stream_begin(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let message_id = rtmp_context.get_message_id().unwrap();
        let mut buffer = ByteBuffer::default();
        buffer.put_u16_be(StreamBegin::EVENT_TYPE.into());
        buffer.encode(&StreamBegin::new(message_id));
        write_chunk(self.0.as_mut(), rtmp_context, StreamBegin::CHANNEL.into(), Duration::default(), StreamBegin::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_publisher_status(PublisherStatus::Began);

        info!("Stream Begin got sent.");
        Ok(())
    }

    async fn write_publish_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let publishing_name = rtmp_context.get_publishing_name().unwrap().clone();
        let message_id = rtmp_context.get_message_id().unwrap();
        let information = object!(
            "level" => AmfString::from("status"),
            "code" => AmfString::from("NetStream.Publish.Start"),
            "description" => AmfString::new(format!("{publishing_name} is now published")),
            "details" => publishing_name
        );
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("onStatus"));
        buffer.encode(&Number::from(0));
        buffer.encode(&OnStatus::new(information.clone()));
        write_chunk(self.0.as_mut(), rtmp_context, OnStatus::CHANNEL.into(), Duration::default(), OnStatus::MESSAGE_TYPE, message_id, &Vec::<u8>::from(buffer)).await?;
        rtmp_context.set_information(information);

        rtmp_context.set_publisher_status(PublisherStatus::Published);

        info!("onStatus got sent.");
        Ok(())
    }
}

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for MessageHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        use PublisherStatus::*;

        let basic_header = ready!(pin!(read_basic_header(pin!(self.0.await_until_receiving()))).poll(cx))?;
        let message_header = ready!(pin!(read_message_header(pin!(self.0.await_until_receiving()), basic_header.get_message_format())).poll(cx))?;
        let extended_timestamp = if let Some(timestamp) = message_header.get_timestamp() {
            if timestamp.as_millis() == U24_MAX as u128 {
                let extended_timestamp = ready!(pin!(read_extended_timestamp(pin!(self.0.await_until_receiving()))).poll(cx))?;
                Some(extended_timestamp)
            } else {
                None
            }
        } else {
            None
        };

        let chunk_id = basic_header.get_chunk_id();
        if let Some(last_received_chunk) = rtmp_context.get_last_received_chunk_mut(&chunk_id) {
            if let Some(extended_timestamp) = extended_timestamp {
                last_received_chunk.set_timestamp(extended_timestamp);
            } else {
                message_header.get_timestamp().map(
                    |timestamp| last_received_chunk.set_timestamp(timestamp)
                );
            }
            message_header.get_message_length().map(
                |message_length| last_received_chunk.set_message_length(message_length)
            );
            message_header.get_message_type().map(
                |message_type| last_received_chunk.set_message_type(message_type)
            );
            message_header.get_message_id().map(
                |message_id| last_received_chunk.set_message_id(message_id)
            );
        } else {
            rtmp_context.insert_received_chunk(
                chunk_id,
                LastChunk::new(
                    message_header.get_timestamp().unwrap(),
                    message_header.get_message_length().unwrap(),
                    message_header.get_message_type().unwrap(),
                    message_header.get_message_id().unwrap()
                )
            );
        } 
        let data = ready!(
            pin!(
                read_chunk_data(
                    pin!(self.0.await_until_receiving()),
                    rtmp_context.get_receiving_chunk_size(),
                    rtmp_context.get_last_received_chunk(&chunk_id).unwrap().get_message_length()
                )
            ).poll(cx)
        )?;
        let buffer: ByteBuffer = data.into();

        let message_type = rtmp_context.get_last_received_chunk(&chunk_id).unwrap().get_message_type();
        match message_type {
            MessageType::Acknowledgement => ready!(pin!(self.handle_acknowledgement(rtmp_context, buffer)).poll(cx))?,
            MessageType::Audio | MessageType::Video | MessageType::Data => {
                let timestamp = rtmp_context.get_last_received_chunk(&chunk_id).unwrap().get_timestamp();
                ready!(pin!(self.handle_flv(rtmp_context, buffer, message_type, timestamp)).poll(cx))?
            },
            MessageType::Command => ready!(pin!(self.handle_command_request(rtmp_context, buffer)).poll(cx))?,
            other => unimplemented!("Undefined Message: {other:?}")
        }

        if let Some(publisher_status) = rtmp_context.get_publisher_status() {
            match publisher_status {
                Connected => pin!(self.write_release_stream_response(rtmp_context)).poll(cx),
                Released => pin!(self.write_fc_publish_response(rtmp_context)).poll(cx),
                FcPublished => pin!(self.write_create_stream_response(rtmp_context)).poll(cx),
                Created => {
                    ready!(pin!(self.write_stream_begin(rtmp_context)).poll(cx))?;
                    pin!(self.write_publish_response(rtmp_context)).poll(cx)
                },
                _ => {
                    /* Just receiving flv after publishing. */
                    Poll::Ready(Ok(()))
                }
            }
        } else {
            pin!(self.write_connect_response(rtmp_context)).poll(cx)
        }
    }
}

#[doc(hidden)]
fn handle_message<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> MessageHandler<'a, RW> {
    MessageHandler(stream)
}

#[doc(hidden)]
#[derive(Debug)]
struct CloseHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> CloseHandler<'_, RW> {
    async fn write_fc_unpublish_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let playpath = rtmp_context.get_playpath().unwrap().clone();
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("FCUnpublish"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&FcUnpublish::new(playpath));
        write_chunk(self.0.as_mut(), rtmp_context, FcUnpublish::CHANNEL.into(), Duration::default(), FcUnpublish::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        info!("FCUnpublish got sent.");
        Ok(())
    }

    async fn write_delete_stream_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let message_id = rtmp_context.get_message_id().unwrap();
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("deleteStream"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&DeleteStream::new(message_id.into()));
        write_chunk(self.0.as_mut(), rtmp_context, DeleteStream::CHANNEL.into(), Duration::default(), DeleteStream::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        info!("deleteStream got sent.");
        Ok(())
    }
}

#[doc(hidden)]
fn handle_close<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> CloseHandler<'a, RW> {
    CloseHandler(stream)
}

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> ErrorHandler for CloseHandler<'_, RW> {
    fn poll_handle_error(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext, error: IOError) -> Poll<IOResult<()>> {
        if error.kind() != ErrorKind::Other {
            if let Some(publisher_status) = rtmp_context.get_publisher_status() {
                if publisher_status >= PublisherStatus::FcPublished {
                    ready!(pin!(self.write_fc_unpublish_request(rtmp_context)).poll(cx))?;
                }

                if publisher_status >= PublisherStatus::Created {
                    ready!(pin!(self.write_delete_stream_request(rtmp_context)).poll(cx))?;
                }
            }
        }

        self.0.as_mut().poll_shutdown(cx)
    }
}

/// The default RTMP handler.
///
/// This handles the raw RTMP by well-known communication steps. That is, this performs just following steps.
///
/// 1. Checks the application name from the [`Connect`] command.
/// 2. Checks the playpath from the [`ReleaseStream`]/[`FcPublish`] command.
/// 3. Provides a message ID when receives the [`CreateStream`] command.
/// 4. Checks publication informations from the [`Publish`] command.
/// 5. Then receives FLV media data.
///
/// If receiving data size exceeds server's bandwidth, this reports its thing via the [`Acknowledgement`] message to its client.
/// And if some error occurs in any step, sends commands which are [`FcUnpublish`] and [`DeleteStream`] to its client, then terminates its connection.
/// These perform to delete the playpath and a message ID from its context.
/// However also these can be sent from clients.
///
/// # Examples
///
/// ```rust
/// use std::marker::PhantomData;
/// use sheave_core::handlers::{
///     RtmpContext,
///     VecStream
/// };
/// use sheave_server::{
///     Server,
///     handlers::RtmpHandler,
/// };
///
/// let stream = VecStream::default();
/// let rtmp_context = RtmpContext::default();
/// let server = Server::new(stream, rtmp_context, PhantomData::<RtmpHandler<VecStream>>);
/// ```
///
/// [`Connect`]: sheave_core::messages::Connect
/// [`ReleaseSream`]: sheave_core::messages::ReleaseStream
/// [`FcPublish`]: sheave_core::messages::FcPublish
/// [`CreateStream`]: sheave_core::messages::CreateStream
/// [`Publish`]: sheave_core::messages::Publish
/// [`Acknowledgement`]: sheave_core::messages::Acknowledgement
/// [`FcUnpublish`]: sheave_core::messages::FcUnpublish
/// [`DeleteStream`]: sheave_core::messages::DeleteStream
#[derive(Debug)]
pub struct RtmpHandler<RW: AsyncRead + AsyncWrite + Unpin>(Arc<StreamWrapper<RW>>);

impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for RtmpHandler<RW> {
    fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        pin!(
            handle_handshake(self.0.make_weak_pin())
                .while_ok(handle_message(self.0.make_weak_pin()).wrap(write_acknowledgement(self.0.make_weak_pin())))
                .map_err(handle_close(self.0.make_weak_pin()))
        ).poll_handle(cx, rtmp_context)
    }
}

impl<RW: AsyncRead + AsyncWrite + Unpin> HandlerConstructor<StreamWrapper<RW>> for RtmpHandler<RW> {
    fn new(stream: Arc<StreamWrapper<RW>>) -> Self {
        Self(stream)
    }
}
