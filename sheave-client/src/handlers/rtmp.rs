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
use futures::{
    TryStreamExt,
    ready
};
use tokio::{
    io::{
        AsyncRead,
        AsyncWrite
    },
    time::timeout
};
use sheave_core::{
    ByteBuffer,
    Decoder,
    Encoder,
    U24_MAX,
    flv::tags::*,
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
        EncryptionAlgorithm,
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
        EventType,
        FcPublish,
        FcUnpublish,
        OnFcPublish,
        OnStatus,
        Publish,
        ReleaseStream,
        ReleaseStreamResult,
        StreamBegin,
        Audio,
        Video,
        SetDataFrame,
        amf::v0::{
            Number,
            AmfString,
            Object
        },
        headers::MessageType
    },
    object,
    readers::*,
    writers::*
};
use super::{
    connection_error,
    publication_error,
    middlewares::write_acknowledgement
};

#[doc(hidden)]
#[derive(Debug)]
struct HandshakeHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> HandshakeHandler<'_, RW> {
    async fn handle_first_handshake(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
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

        write_encryption_algorithm(self.0.as_mut(), encryption_algorithm).await?;
        write_handshake(self.0.as_mut(), &client_request).await?;

        rtmp_context.set_encryption_algorithm(encryption_algorithm);
        rtmp_context.set_client_handshake(client_request);

        Ok(())
    }

    async fn handle_second_handshake(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let encryption_algorithm = read_encryption_algorithm(self.0.as_mut()).await?;
        let mut server_request = read_handshake(self.0.as_mut()).await?;
        let server_response = read_handshake(self.0.as_mut()).await?;

        if !rtmp_context.is_signed() {
            write_handshake(self.0.as_mut(), &server_request).await?;

            rtmp_context.set_server_handshake(server_request);
            rtmp_context.set_client_handshake(server_response);

            Ok(())
        } else if !server_request.did_digest_match(encryption_algorithm, Handshake::SERVER_KEY) {
            Err(inconsistent_sha(server_response.get_digest(encryption_algorithm).to_vec()))
        } else {
            let mut server_response_key: Vec<u8> = Vec::new();
            server_response_key.extend_from_slice(Handshake::SERVER_KEY);
            server_response_key.extend_from_slice(Handshake::COMMON_KEY);

            if !server_response.did_signature_match(encryption_algorithm, &server_response_key) {
                Err(inconsistent_sha(server_response.get_signature().to_vec()))
            } else {
                let mut client_response_key: Vec<u8> = Vec::new();
                client_response_key.extend_from_slice(Handshake::CLIENT_KEY);
                client_response_key.extend_from_slice(Handshake::COMMON_KEY);
                server_request.imprint_signature(encryption_algorithm, &client_response_key);
                write_handshake(self.0.as_mut(), &server_request).await?;

                rtmp_context.set_signed(true);
                rtmp_context.set_server_handshake(server_request);
                rtmp_context.set_client_handshake(server_response);

                Ok(())
            }
        }
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
    async fn write_connect_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let connect = Connect::new(
            object!(
                "app" => rtmp_context.get_app().unwrap().clone(),
                "type" => AmfString::from("nonprivate"),
                "flashVer" => AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"),
                "tcUrl" => rtmp_context.get_tc_url().unwrap().clone()
            )
        );
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("connect"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&connect);
        write_chunk(self.0.as_mut(), rtmp_context, Connect::CHANNEL.into(), Duration::default(), Connect::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_command_object(connect.into());

        Ok(())
    }

    async fn write_release_stream_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("releaseStream"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&ReleaseStream::new(rtmp_context.get_playpath().unwrap().clone()));
        write_chunk(self.0.as_mut(), rtmp_context, ReleaseStream::CHANNEL.into(), Duration::default(), ReleaseStream::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        Ok(())
    }

    async fn write_fc_publish_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("FCPublish"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&FcPublish::new(rtmp_context.get_playpath().unwrap().clone()));
        write_chunk(self.0.as_mut(), rtmp_context, FcPublish::CHANNEL.into(), Duration::default(), FcPublish::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        Ok(())
    }

    async fn write_create_stream_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("createStream"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&CreateStream);
        write_chunk(self.0.as_mut(), rtmp_context, CreateStream::CHANNEL.into(), Duration::default(), CreateStream::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        Ok(())
    }

    async fn write_publish_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let publishing_name = rtmp_context.get_playpath().unwrap().clone();
        let publishing_type = "live";
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("publish"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&Publish::new(publishing_name.clone(), publishing_type.into()));
        let message_id = rtmp_context.get_message_id().unwrap();
        write_chunk(self.0.as_mut(), rtmp_context, Publish::CHANNEL.into(), Duration::default(), Publish::MESSAGE_TYPE, message_id, &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_publishing_name(publishing_name.clone());
        rtmp_context.set_publishing_type(publishing_type.into());

        Ok(())
    }

    async fn write_flv(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let flv_tag = rtmp_context.get_input_mut().unwrap().try_next().await?;

        if let Some(flv_tag) = flv_tag {
            let message_id = rtmp_context.get_message_id().unwrap();
            let channel;
            let message_type;

            match flv_tag.get_tag_type() {
                TagType::Audio => {
                    channel = Audio::CHANNEL;
                    message_type = Audio::MESSAGE_TYPE;
                },
                TagType::Video => {
                    channel = Video::CHANNEL;
                    message_type = Video::MESSAGE_TYPE;
                },
                TagType::ScriptData => {
                    channel = SetDataFrame::CHANNEL;
                    message_type = SetDataFrame::MESSAGE_TYPE;
                },
                _ => unreachable!("We never get other type from FlvTag::get_tag_type().")
            }

            let timestamp = flv_tag.get_timestamp();
            let mut buffer = ByteBuffer::default();
            buffer.encode(&flv_tag);

            write_chunk(self.0.as_mut(), rtmp_context, channel.into(), timestamp, message_type, message_id, &Vec::<u8>::from(buffer)).await
        } else {
            Ok(())
        }
    }

    fn handle_acknowledgement(&mut self, _: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<Acknowledgement>::decode(&mut buffer)?;
        Ok(())
    }

    fn handle_stream_begin(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<StreamBegin>::decode(&mut buffer)?;

        rtmp_context.set_publisher_status(PublisherStatus::Began);

        Ok(())
    }

    fn handle_user_control(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use EventType::*;

        let event_type: EventType = buffer.get_u16_be()?.into();
        match event_type {
            StreamBegin => self.handle_stream_begin(rtmp_context, buffer)?,
            _ => unreachable!("Publisher gets just a Stream Begin event.")
        }

        Ok(())
    }

    fn handle_connect_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer, command: AmfString) -> IOResult<()> {
        let response: ConnectResult = buffer.decode()?;
        let (properties, information): (Object, Object) = response.into();

        if "_error" == command {
            return Err(connection_error(information))
        }

        /* Something logger here. */

        rtmp_context.set_properties(properties);
        rtmp_context.set_information(information);

        rtmp_context.set_publisher_status(PublisherStatus::Connected);

        Ok(())
    }

    fn handle_release_stream_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer, _: AmfString) -> IOResult<()> {
        Decoder::<ReleaseStreamResult>::decode(&mut buffer)?;

        rtmp_context.set_publisher_status(PublisherStatus::Released);

        Ok(())
    }

    fn handle_fc_publish_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer, _: AmfString) -> IOResult<()> {
        Decoder::<OnFcPublish>::decode(&mut buffer)?;

        rtmp_context.set_publisher_status(PublisherStatus::FcPublished);

        Ok(())
    }

    fn handle_create_stream_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer, _: AmfString) -> IOResult<()> {
        let response: CreateStreamResult = buffer.decode()?;
        let message_id: u32 = response.into();
        rtmp_context.set_message_id(message_id);

        rtmp_context.set_publisher_status(PublisherStatus::Created);

        Ok(())
    }

    fn handle_publish_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer, _: AmfString) -> IOResult<()> {
        let response: OnStatus = buffer.decode()?;
        let information: Object = response.into();

        if information.get_properties()["level"] == AmfString::from("error") {
            return Err(publication_error(information))
        }

        /* Something logger here. */

        rtmp_context.set_information(information);

        rtmp_context.set_publisher_status(PublisherStatus::Published);

        Ok(())
    }

    fn handle_fc_unpublish_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer, _: AmfString) -> IOResult<()> {
        Decoder::<FcUnpublish>::decode(&mut buffer)?;
        rtmp_context.reset_playpath();

        Ok(())
    }

    fn handle_delete_stream_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer, _: AmfString) -> IOResult<()> {
        Decoder::<DeleteStream>::decode(&mut buffer)?;
        rtmp_context.reset_message_id();

        Ok(())
    }

    fn handle_command_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use PublisherStatus::*;

        let command: AmfString = buffer.decode()?;
        // NOTE: Currently unused but exists.
        Decoder::<Number>::decode(&mut buffer)?;

        if command == "FCUnpublish" {
            return self.handle_fc_unpublish_request(rtmp_context, buffer, command)
        } else if command == "deleteStream" {
            return self.handle_delete_stream_request(rtmp_context, buffer, command)
        } else {
            /* In this step, does nothing unless command is either "FCUnpublish" or "deleteStream". */
        }

        if let Some(publisher_status) = rtmp_context.get_publisher_status() {
            match publisher_status {
                Connected => self.handle_release_stream_response(rtmp_context, buffer, command),
                Released => self.handle_fc_publish_response(rtmp_context, buffer, command),
                FcPublished => self.handle_create_stream_response(rtmp_context, buffer, command),
                Created => self.handle_publish_response(rtmp_context, buffer, command),
                _ => Ok(())
            }
        } else {
            self.handle_connect_response(rtmp_context, buffer, command)
        }
    }
}

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for MessageHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        use PublisherStatus::*;
        use MessageType::*;

        if let Some(publisher_status) = rtmp_context.get_publisher_status() {
            match publisher_status {
                Connected => ready!(pin!(self.write_release_stream_request(rtmp_context)).poll(cx))?,
                Released => ready!(pin!(self.write_fc_publish_request(rtmp_context)).poll(cx))?,
                FcPublished => ready!(pin!(self.write_create_stream_request(rtmp_context)).poll(cx))?,
                Created => ready!(pin!(self.write_publish_request(rtmp_context)).poll(cx))?,
                Published => ready!(pin!(self.write_flv(rtmp_context)).poll(cx))?,
                _ => {}
            }
        } else {
            ready!(pin!(self.write_connect_request(rtmp_context)).poll(cx))?;
        }

        // NOTE:
        //  Basically, we receive nothing while sending FLV data.
        //  However server may sned either Acknowledgement or FCUnpublish/deleteStream.
        //  This timeout considers when we get something above.
        let basic_header = if let Some(Published) = rtmp_context.get_publisher_status() {
            if let Ok(result) = ready!(pin!(timeout(rtmp_context.get_timeout_duration(), read_basic_header(self.0.as_mut()))).poll(cx)) {
                result?
            } else {
                return Poll::Pending
            }
        } else {
            ready!(pin!(read_basic_header(self.0.as_mut())).poll(cx))?
        };
        let message_header = ready!(pin!(read_message_header(self.0.as_mut(), basic_header.get_message_format())).poll(cx))?;
        let extended_timestamp = if let Some(timestamp) = message_header.get_timestamp() {
            if timestamp.as_millis() == U24_MAX as u128 {
                let extended_timestamp = ready!(pin!(read_extended_timestamp(self.0.as_mut())).poll(cx))?;
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
                if let Some(timestamp) = message_header.get_timestamp() {
                    last_received_chunk.set_timestamp(timestamp);
                }
            }

            if let Some(message_length) = message_header.get_message_length() {
                last_received_chunk.set_message_length(message_length);
            }

            if let Some(message_type) = message_header.get_message_type() {
                last_received_chunk.set_message_type(message_type);
            }

            if let Some(message_id) = message_header.get_message_id() {
                last_received_chunk.set_message_id(message_id);
            }
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

        let message_length = rtmp_context.get_last_received_chunk(&chunk_id).unwrap().get_message_length();
        let receiving_chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = ready!(pin!(read_chunk_data(self.0.as_mut(), receiving_chunk_size, message_length)).poll(cx))?;
        let buffer: ByteBuffer = data.into();

        let message_type = rtmp_context.get_last_received_chunk(&chunk_id).unwrap().get_message_type();
        match message_type {
            Acknowledgement => self.handle_acknowledgement(rtmp_context, buffer)?,
            UserControl => self.handle_user_control(rtmp_context, buffer)?,
            Command => self.handle_command_response(rtmp_context, buffer)?,
            _ => unreachable!("Publisher gets just messages which are the Acknowledgement, the User Control and Command.")
        }

        Poll::Ready(Ok(()))
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
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("FCUnpublish"));
        buffer.encode(&FcUnpublish::new(rtmp_context.get_playpath().unwrap().clone()));
        write_chunk(self.0.as_mut(), rtmp_context, FcUnpublish::CHANNEL.into(), Duration::default(), FcUnpublish::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await
    }

    async fn write_delete_stream_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let message_id = rtmp_context.get_message_id().unwrap();

        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("deleteStream"));
        buffer.encode(&DeleteStream::new(message_id.into()));
        write_chunk(self.0.as_mut(), rtmp_context, DeleteStream::CHANNEL.into(), Duration::default(), DeleteStream::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await
    }
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

#[doc(hidden)]
fn handle_close<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> CloseHandler<'a, RW> {
    CloseHandler(stream)
}

/// The default RTMP handler.
///
/// This handles the raw RTMP by well-known communication steps. That is, this performs just following steps.
///
/// 1. Specifies the application name via the [`Connect`] command.
/// 2. Specifies the playpath via the [`ReleaseStream`]/[`FcPublish`] command.
/// 3. Requests a message ID via the [`CreateStream`] command.
/// 4. Specifies publication informations via the [`Publish`] command.
/// 5. Then sends FLV media data.
///
/// If receiving data size exceeds client's bandwidth, this reports its thing via the [`Acknowledgement`] message to its server.
/// And if some error occurs in any step, sends commands which are [`FcUnpublish`] and [`DeleteStream`] to its server, then terminates its connection.
/// These perform to delete the playpath and a message ID from its context.
/// However also these can be sent from servers.
///
/// # Examples
///
/// ```rust
/// use std::marker::PhantomData;
/// use sheave_core::handlers::{
///     RtmpContext,
///     VecStream
/// };
/// use sheave_client::{
///     Client,
///     handlers::RtmpHandler,
/// };
///
/// let stream = VecStream::default();
/// let rtmp_context = RtmpContext::default();
/// let client = Client::new(stream, rtmp_context, PhantomData::<RtmpHandler<VecStream>>);
/// ```
///
/// [`Connect`]: sheave_core::messages::Connect
/// [`ReleaseSream`]: sheave_core::messages::ReleaseStream
/// [`FcPublish`]: sheave_core::messages::FcPublish
/// [`CreateStream`]: sheave_core::messages::CreateStream
/// [`Publish`]: sheave_core::messages::Publish
/// [`Acknowledgement`]: sheave_core::messages::Acknowledgement
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
