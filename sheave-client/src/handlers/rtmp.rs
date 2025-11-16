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
    error,
    info
};
use futures::ready;
use tokio::io::{
    AsyncRead,
    AsyncWrite
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
        ClientType,
        ErrorHandler,
        HandlerConstructor,
        LastChunk,
        PublisherStatus,
        RtmpContext,
        StreamWrapper,
        SubscriberStatus,
        inconsistent_sha,
        stream_got_exhausted
    },
    handshake::{
        EncryptionAlgorithm,
        Handshake,
        Version
    },
    messages::{
        /* Used in common */
        Channel,
        ChunkData,
        Connect,
        ConnectResult,
        CreateStream,
        CreateStreamResult,
        UserControl,
        EventType,
        OnStatus,
        Audio,
        Video,
        SetDataFrame,
        Acknowledgement,
        amf::v0::{
            Number,
            AmfString,
            Object
        },
        headers::MessageType,

        /* Publisher-side */
        ReleaseStream,
        ReleaseStreamResult,
        FcPublish,
        OnFcPublish,
        StreamBegin,
        Publish,
        FcUnpublish,
        DeleteStream,

        /* Subscriber-side */
        WindowAcknowledgementSize,
        FcSubscribe,
        SetBufferLength,
        Play,
        amf::v0::Boolean
    },
    net::RtmpReadExt,
    object,
    readers::*,
    writers::*
};
use super::{
    error_response,
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

        info!("First handshake got handled.");
        Ok(())
    }

    async fn handle_second_handshake(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let encryption_algorithm = read_encryption_algorithm(pin!(self.0.await_until_receiving())).await?;
        let mut server_request = read_handshake(pin!(self.0.await_until_receiving())).await?;
        let server_response = read_handshake(pin!(self.0.await_until_receiving())).await?;

        if !rtmp_context.is_signed() {
            write_handshake(self.0.as_mut(), &server_request).await?;

            rtmp_context.set_server_handshake(server_request);
            rtmp_context.set_client_handshake(server_response);

        } else if !server_request.did_digest_match(encryption_algorithm, Handshake::SERVER_KEY) {
            error!("Invalid SHA digest/signature: {:x?}", server_request.get_digest(encryption_algorithm));
            return Err(inconsistent_sha(server_response.get_digest(encryption_algorithm).to_vec()))
        } else {
            let mut server_response_key: Vec<u8> = Vec::new();
            server_response_key.extend_from_slice(Handshake::SERVER_KEY);
            server_response_key.extend_from_slice(Handshake::COMMON_KEY);

            if !server_response.did_signature_match(encryption_algorithm, &server_response_key) {
                error!("Invalid SHA digest/signature: {:x?}", server_response.get_signature());
                return Err(inconsistent_sha(server_response.get_signature().to_vec()))
            } else {
                let mut client_response_key: Vec<u8> = Vec::new();
                client_response_key.extend_from_slice(Handshake::CLIENT_KEY);
                client_response_key.extend_from_slice(Handshake::COMMON_KEY);
                server_request.imprint_signature(encryption_algorithm, &client_response_key);
                write_handshake(self.0.as_mut(), &server_request).await?;

                rtmp_context.set_server_handshake(server_request);
                rtmp_context.set_client_handshake(server_response);
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
    async fn write_connect_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        use ClientType::*;

        let client_type = rtmp_context.get_client_type().unwrap();

        rtmp_context.increase_transaction_id();

        let command_object = match client_type {
            Publisher => object!(
                "app" => rtmp_context.get_app().unwrap().clone(),
                "type" => AmfString::from("nonprivate"),
                "flashVer" => AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"),
                "tcUrl" => rtmp_context.get_tc_url().unwrap().clone()
            ),
            Subscriber => object!(
                "app" => rtmp_context.get_app().unwrap().clone(),
                "flashVer" => AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"),
                "tcUrl" => rtmp_context.get_tc_url().unwrap().clone(),
                "fpad" => Boolean::new(0),
                "capabilities" => Number::from(15u8),
                "audioCodecs" => Number::from(4071u16),
                "videoCodecs" => Number::from(252u8),
                "videoFunction" => Number::from(1u8)
            )
        };
        let connect = Connect::new(command_object);
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("connect"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&connect);
        write_chunk(self.0.as_mut(), rtmp_context, Connect::CHANNEL.into(), Duration::default(), Connect::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_command_object(connect.into());

        info!("connect got sent.");
        Ok(())
    }

    async fn write_window_acknowledgement_size(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&rtmp_context.get_window_acknowledgement_size());
        write_chunk(self.0.as_mut(), rtmp_context, WindowAcknowledgementSize::CHANNEL.into(), Duration::default(), WindowAcknowledgementSize::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_subscriber_status(SubscriberStatus::WindowAcknowledgementSizeGotSent);

        info!("Window Acknowledgement Size got sent.");
        Ok(())
    }

    async fn write_release_stream_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("releaseStream"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&ReleaseStream::new(rtmp_context.get_topic_id().unwrap().clone()));
        write_chunk(self.0.as_mut(), rtmp_context, ReleaseStream::CHANNEL.into(), Duration::default(), ReleaseStream::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        info!("releaseStream got sent.");
        Ok(())
    }

    async fn write_fc_publish_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("FCPublish"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&FcPublish::new(rtmp_context.get_topic_id().unwrap().clone()));
        write_chunk(self.0.as_mut(), rtmp_context, FcPublish::CHANNEL.into(), Duration::default(), FcPublish::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        info!("FCPublish got sent.");
        Ok(())
    }

    async fn write_create_stream_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("createStream"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&CreateStream);
        write_chunk(self.0.as_mut(), rtmp_context, CreateStream::CHANNEL.into(), Duration::default(), CreateStream::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        info!("createStream got sent.");
        Ok(())
    }

    async fn write_fc_subscribe_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let topic_id = rtmp_context.get_topic_id().unwrap().clone();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&FcSubscribe::new(topic_id));
        write_chunk(self.0.as_mut(), rtmp_context, FcSubscribe::CHANNEL.into(), Duration::default(), FcSubscribe::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_subscriber_status(SubscriberStatus::FcSubscribed);

        info!("FCSubscribe got sent.");
        Ok(())
    }

    async fn write_publish_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let publishing_name = rtmp_context.get_topic_id().unwrap().clone();
        let publishing_type = "live";
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("publish"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&Publish::new(publishing_name.clone(), publishing_type.into()));
        let message_id = rtmp_context.get_message_id().unwrap();
        write_chunk(self.0.as_mut(), rtmp_context, Publish::CHANNEL.into(), Duration::default(), Publish::MESSAGE_TYPE, message_id, &Vec::<u8>::from(buffer)).await?;

        info!("publish got sent.");
        Ok(())
    }

    async fn write_play_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        rtmp_context.increase_transaction_id();

        let stream_name = rtmp_context.get_topic_id().unwrap().clone();
        let play_mode = rtmp_context.get_play_mode().unwrap();
        let start_time: Number = if let Some(start_time) = rtmp_context.get_start_time() {
            Number::new(start_time.as_millis() as u64 as f64)
        } else {
            Number::new((1000 * play_mode as i64) as f64)
        };

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("play"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&Play::new(stream_name.clone(), start_time));
        write_chunk(self.0.as_mut(), rtmp_context, Play::CHANNEL.into(), Duration::default(), Play::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        info!("play got sent.");
        Ok(())
    }

    async fn write_buffer_length(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let message_id = rtmp_context.get_message_id().unwrap();
        let mut buffer = ByteBuffer::default();
        buffer.put_u16_be(SetBufferLength::EVENT_TYPE.into());
        buffer.encode(&SetBufferLength::new(message_id, rtmp_context.get_buffer_length()));
        write_chunk(self.0.as_mut(), rtmp_context, SetBufferLength::CHANNEL.into(), Duration::default(), SetBufferLength::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_subscriber_status(SubscriberStatus::BufferLengthGotSent);

        info!("Buffer Length got sent.");
        Ok(())
    }

    async fn write_flv(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        for next in rtmp_context.get_topic_mut().unwrap() {
            let flv_tag = next?;
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
                TagType::Other => {
                    channel = Channel::Other;
                    message_type = MessageType::Other;
                }
            }
            let timestamp = flv_tag.get_timestamp();
            let data: Vec<u8> = if let MessageType::Data = message_type {
                let mut buffer = ByteBuffer::default();
                buffer.encode(&AmfString::from("@setDataFrame"));
                buffer.put_bytes(flv_tag.get_data());
                buffer.into()
            } else {
                flv_tag.get_data().to_vec()
            };
            write_chunk(self.0.as_mut(), rtmp_context, channel.into(), timestamp, message_type, message_id, &data).await?;

            info!("FLV chunk got sent.");
            return Ok(())
        }

        // NOTE: Default return value when no FLV tag exists.
        info!("FLV data became empty.");
        Err(stream_got_exhausted())
    }

    async fn handle_acknowledgement(&mut self, _: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<Acknowledgement>::decode(&mut buffer)?;

        info!("Acknowledgement got handled.");
        Ok(())
    }

    async fn handle_stream_begin(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use ClientType::*;

        let client_type = rtmp_context.get_client_type().unwrap();

        Decoder::<StreamBegin>::decode(&mut buffer)?;

        match client_type {
            Publisher => rtmp_context.set_publisher_status(PublisherStatus::Began),
            Subscriber => rtmp_context.set_subscriber_status(SubscriberStatus::Began)
        }

        info!("Stream Begin got handled.");
        Ok(())
    }

    async fn handle_user_control(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use EventType::*;

        let event_type: EventType = buffer.get_u16_be()?.into();
        match event_type {
            StreamBegin => self.handle_stream_begin(rtmp_context, buffer).await,
            _ => unreachable!("Publisher gets just a Stream Begin event.")
        }
    }

    async fn handle_error_response(&mut self, rtmp_context: &mut RtmpContext, information: Object) -> IOResult<()> {
        let error = error_response(information.clone());
        rtmp_context.set_information(information);

        error!("{error}");
        Err(error)
    }

    async fn handle_connect_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use ClientType::*;

        let response: ConnectResult = buffer.decode()?;
        let (properties, information): (Object, Object) = response.into();

        rtmp_context.set_properties(properties);
        rtmp_context.set_information(information);

        match rtmp_context.get_client_type().unwrap() {
            Publisher => rtmp_context.set_publisher_status(PublisherStatus::Connected),
            Subscriber => rtmp_context.set_subscriber_status(SubscriberStatus::Connected)
        }

        info!("connect result got handled.");
        Ok(())
    }

    async fn handle_release_stream_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<ReleaseStreamResult>::decode(&mut buffer)?;

        rtmp_context.set_publisher_status(PublisherStatus::Released);

        info!("releaseStream result got handled.");
        Ok(())
    }

    async fn handle_fc_publish_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<OnFcPublish>::decode(&mut buffer)?;

        rtmp_context.set_publisher_status(PublisherStatus::FcPublished);

        info!("onFCPublish got handled.");
        Ok(())
    }

    async fn handle_create_stream_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use ClientType::*;

        let client_type = rtmp_context.get_client_type().unwrap();

        let response: CreateStreamResult = buffer.decode()?;
        let message_id: u32 = response.into();
        rtmp_context.set_message_id(message_id);

        match client_type {
            Publisher => rtmp_context.set_publisher_status(PublisherStatus::Created),
            Subscriber => rtmp_context.set_subscriber_status(SubscriberStatus::Created)
        }

        info!("createStream result got handled.");
        Ok(())
    }

    async fn handle_publish_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let response: OnStatus = buffer.decode()?;
        let information: Object = response.into();

        /*
         *  NOTE:
         *      Some error in publication step is checkable only by information the field.
         *      Because the publish command doesn't have _error command.
         */
        if information.get_properties()["level"] == AmfString::from("error") {
            return self.handle_error_response(rtmp_context, information).await
        }

        rtmp_context.set_information(information);

        rtmp_context.set_publisher_status(PublisherStatus::Published);

        info!("onStatus(publish) got handled.");
        Ok(())
    }

    async fn handle_fc_unpublish_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<FcUnpublish>::decode(&mut buffer)?;
        rtmp_context.reset_topic_id();

        info!("FCUnpublish got handled.");
        Ok(())
    }

    async fn handle_delete_stream_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<DeleteStream>::decode(&mut buffer)?;
        rtmp_context.reset_message_id();

        info!("deleteStream got handled.");
        Ok(())
    }

    async fn handle_publisher_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use PublisherStatus::*;

        let command: AmfString = buffer.decode()?;

        // NOTE: onFCPublish has no transaction ID.
        if command != "onFCPublish" {
            // NOTE: Otherwise, currently unused but exists.
            Decoder::<Number>::decode(&mut buffer)?;
        }

        if command == "FCUnpublish" {
            return self.handle_fc_unpublish_request(rtmp_context, buffer).await
        } else if command == "deleteStream" {
            return self.handle_delete_stream_request(rtmp_context, buffer).await
        } else if command == "_error" {
            let information: Object = buffer.decode()?;
            return self.handle_error_response(rtmp_context, information).await
        } else {
            /* In this step, does nothing unless command is either "FCUnpublish" or "deleteStream". */
        }

        if let Some(publisher_status) = rtmp_context.get_publisher_status() {
            match publisher_status {
                Connected => self.handle_release_stream_response(rtmp_context, buffer).await,
                Released => self.handle_fc_publish_response(rtmp_context, buffer).await,
                FcPublished => self.handle_create_stream_response(rtmp_context, buffer).await,
                Began => self.handle_publish_response(rtmp_context, buffer).await,
                _ => Ok(())
            }
        } else {
            self.handle_connect_response(rtmp_context, buffer).await
        }
    }

    async fn handle_play_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let response: OnStatus = buffer.decode()?;
        let information: Object = response.into();

        /*
         *  NOTE:
         *      Some error in subscription step is checkable only by information the field.
         *      Because the play command doesn't have _error command.
         */
        if information.get_properties()["level"] == AmfString::from("error") {
            return self.handle_error_response(rtmp_context, information).await
        }

        rtmp_context.set_information(information);

        rtmp_context.set_subscriber_status(SubscriberStatus::Played);

        info!("onStatus(play) got handled.");
        Ok(())
    }

    async fn handle_subscriber_response(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use SubscriberStatus::*;

        let command: AmfString = buffer.decode()?;
        Decoder::<Number>::decode(&mut buffer)?;

        if command == "_error" {
            let information: Object = buffer.decode()?;
            return self.handle_error_response(rtmp_context, information).await
        }

        if let Some(subscriber_status) = rtmp_context.get_subscriber_status() {
            match subscriber_status {
                WindowAcknowledgementSizeGotSent => self.handle_create_stream_response(rtmp_context, buffer).await,
                Began => self.handle_play_response(rtmp_context, buffer).await,
                _ => return Ok(())
            }
        } else {
            self.handle_connect_response(rtmp_context, buffer).await
        }
    }

    async fn handle_command_response(&mut self, rtmp_context: &mut RtmpContext, buffer: ByteBuffer) -> IOResult<()> {
        use ClientType::*;

        match rtmp_context.get_client_type().unwrap() {
            Publisher => self.handle_publisher_response(rtmp_context, buffer).await,
            Subscriber => self.handle_subscriber_response(rtmp_context, buffer).await
        }
    }

    async fn handle_flv(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer, message_type: MessageType, timestamp: Duration) -> IOResult<()> {
        let topic = rtmp_context.get_topic().unwrap();

        let tag_type = match message_type {
            MessageType::Audio => TagType::Audio,
            MessageType::Video => TagType::Video,
            MessageType::Data => TagType::ScriptData,
            _ => TagType::Other
        };

        if let TagType::ScriptData = tag_type {
            Decoder::<AmfString>::decode(&mut buffer)?;
        }

        let data: Vec<u8> = buffer.into();
        let flv_tag = FlvTag::new(tag_type, timestamp, data);
        topic.append_flv_tag(flv_tag)?;

        info!("FLV chunk got handled.");
        Ok(())
    }
}

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for MessageHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        use MessageType::*;

        if let Some(publisher_status) = rtmp_context.get_publisher_status() {
            match publisher_status {
                PublisherStatus::Connected => ready!(pin!(self.write_release_stream_request(rtmp_context)).poll(cx))?,
                PublisherStatus::Released => ready!(pin!(self.write_fc_publish_request(rtmp_context)).poll(cx))?,
                PublisherStatus::FcPublished => ready!(pin!(self.write_create_stream_request(rtmp_context)).poll(cx))?,
                PublisherStatus::Created => ready!(pin!(self.write_publish_request(rtmp_context)).poll(cx))?,
                PublisherStatus::Published => ready!(pin!(self.write_flv(rtmp_context)).poll(cx))?,
                _ => {}
            }
        } else if let Some(subscriber_status) = rtmp_context.get_subscriber_status() {
            match subscriber_status {
                SubscriberStatus::Connected => {
                    ready!(pin!(self.write_window_acknowledgement_size(rtmp_context)).poll(cx))?;
                    ready!(pin!(self.write_create_stream_request(rtmp_context)).poll(cx))?
                },
                SubscriberStatus::Created => {
                    ready!(pin!(self.write_fc_subscribe_request(rtmp_context)).poll(cx))?;
                    rtmp_context.set_subscriber_status(SubscriberStatus::AdditionalCommandGotSent);
                },
                SubscriberStatus::AdditionalCommandGotSent => {
                    ready!(pin!(self.write_play_request(rtmp_context)).poll(cx))?;
                    ready!(pin!(self.write_buffer_length(rtmp_context)).poll(cx))?
                },
                _ => {}
            }
        } else {
            ready!(pin!(self.write_connect_request(rtmp_context)).poll(cx))?;
        }

        let basic_header = if let Some(PublisherStatus::Published) = rtmp_context.get_publisher_status() {
            ready!(pin!(read_basic_header(pin!(self.0.try_read_after(rtmp_context.get_await_duration().unwrap())))).poll(cx))?
        } else {
            ready!(pin!(read_basic_header(pin!(self.0.await_until_receiving()))).poll(cx))?
        };
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
        let data = ready!(pin!(read_chunk_data(pin!(self.0.await_until_receiving()), receiving_chunk_size, message_length)).poll(cx))?;
        let buffer: ByteBuffer = data.into();

        let message_type = rtmp_context.get_last_received_chunk(&chunk_id).unwrap().get_message_type();
        match message_type {
            Acknowledgement => pin!(self.handle_acknowledgement(rtmp_context, buffer)).poll(cx),
            UserControl => pin!(self.handle_user_control(rtmp_context, buffer)).poll(cx),
            Command => pin!(self.handle_command_response(rtmp_context, buffer)).poll(cx),
            Audio | Video | Data => {
                let timestamp = rtmp_context.get_last_received_chunk(&chunk_id).unwrap().get_timestamp();
                pin!(self.handle_flv(rtmp_context, buffer, message_type, timestamp)).poll(cx)
            },
            other => unimplemented!("Undefined Message: {other:?}")
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
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("FCUnpublish"));
        buffer.encode(&FcUnpublish::new(rtmp_context.get_topic_id().unwrap().clone()));
        write_chunk(self.0.as_mut(), rtmp_context, FcUnpublish::CHANNEL.into(), Duration::default(), FcUnpublish::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        info!("FCUnpublish got sent.");
        Ok(())
    }

    async fn write_delete_stream_request(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let message_id = rtmp_context.get_message_id().unwrap();

        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("deleteStream"));
        buffer.encode(&DeleteStream::new(message_id.into()));
        write_chunk(self.0.as_mut(), rtmp_context, DeleteStream::CHANNEL.into(), Duration::default(), DeleteStream::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        info!("deleteStream got sent.");
        Ok(())
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
/// This handles the raw RTMP by well-known communication steps, that is, this performs just following steps.
///
/// # As a publisher
///
/// 1. Specifies the application name via the [`Connect`] command.
/// 2. Specifies the topic path via the [`ReleaseStream`]/[`FcPublish`] command.
/// 3. Requests a message ID via the [`CreateStream`] command.
/// 4. Specifies publication informations via the [`Publish`] command.
/// 5. Then sends FLV media data.
///
/// If some error occurs in any step, sends commands which are [`FcUnpublish`] and [`DeleteStream`] to its server, then terminates its connection.
/// These perform to delete the topic path and a message ID from its context.
/// However also these can be sent from servers.
///
/// # As a subscriber
///
/// 1. Specifies the application name via the [`Connect`] command.
/// 2. Tells the size of receiving bandwidth via the [`WindowAcknowledgementSize`] message.
/// 3. Requests a message ID via the [`CreateStream`] command.
/// 4. Specified the topic path via the [`FcSubscribe`] command.
/// 5. Following additional command may be required.
///    * Requests the duration of its topic via the [`GetStreamLength`] command. (in FFmpeg)
///    * Requests a list of topics as a playlist via the [`SetPlaylist`] command. (in OBS)
/// 6. Specifies subscription information via the [`Play`] command.
/// 7. Specifies a time range to buffer its topic via the [`SetBufferLength`] event.
/// 8. Then receives FLV media data.
///
/// If receiving data size exceeds client's bandwidth, this reports its thing via the [`Acknowledgement`] message to its server.
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
/// [`WindowAcknowledgementSize`]: sheave_core::messages::WindowAcknowledgementSize
/// [`FcSubscribe`]: sheave_core::messages::FcSubscribe
/// [`GetStreamLength`]: sheave_core::messages::GetStreamLength
/// [`SetPlaylist`]: sheave_core::messages::SetPlaylist
/// [`Play`]: sheave_core::messages::Play
/// [`SetBufferLength`]: sheave_core::messages::SetBufferLength
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use sheave_core::{
        handlers::VecStream,
        messages::PlayMode
    };
    use super::*;

    #[tokio::test]
    async fn ok_handshake_got_handled() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_signed(true);

        handle_handshake(stream.as_mut()).handle_first_handshake(&mut rtmp_context).await.unwrap();

        let sent_encryption_algorithm = read_encryption_algorithm(stream.as_mut()).await.unwrap();
        let mut sent_client_handshake = read_handshake(stream.as_mut()).await.unwrap();
        assert_eq!(EncryptionAlgorithm::NotEncrypted, sent_encryption_algorithm);
        assert!(sent_client_handshake.did_digest_match(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY));

        let mut stream = pin!(VecStream::default());
        let received_encryption_algorithm = EncryptionAlgorithm::NotEncrypted;
        let mut received_server_handshake = Handshake::new(Instant::now().elapsed(), Version::LATEST_SERVER);
        received_server_handshake.imprint_digest(received_encryption_algorithm, Handshake::SERVER_KEY);
        let mut server_response_key: Vec<u8> = Vec::new();
        server_response_key.extend_from_slice(Handshake::SERVER_KEY);
        server_response_key.extend_from_slice(Handshake::COMMON_KEY);
        sent_client_handshake.imprint_signature(sent_encryption_algorithm, &server_response_key);
        write_encryption_algorithm(stream.as_mut(), received_encryption_algorithm).await.unwrap();
        write_handshake(stream.as_mut(), &received_server_handshake).await.unwrap();
        write_handshake(stream.as_mut(), &sent_client_handshake).await.unwrap();
        assert!(handle_handshake(stream.as_mut()).handle_second_handshake(&mut rtmp_context).await.is_ok());

        let sent_server_handshake = read_handshake(stream.as_mut()).await.unwrap();
        let mut client_response_key: Vec<u8> = Vec::new();
        client_response_key.extend_from_slice(Handshake::CLIENT_KEY);
        client_response_key.extend_from_slice(Handshake::COMMON_KEY);
        assert!(sent_server_handshake.did_signature_match(sent_encryption_algorithm, &client_response_key))
    }

    #[tokio::test]
    async fn ok_publisher_sequence() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_tc_url("");
        rtmp_context.set_app("");
        rtmp_context.set_client_type(ClientType::Publisher);

        handle_message(stream.as_mut()).write_connect_request(&mut rtmp_context).await.unwrap();
        let mut stream = pin!(VecStream::default());
        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &ConnectResult::new(
                object!(
                    "fmsVer" => AmfString::from("FMS/5,0,17"),
                    "capabilities" => Number::new(31f64)
                ),
                object!(
                    "level" => AmfString::from("status"),
                    "code" => AmfString::from("NetConnection.Connect.Success"),
                    "description" => AmfString::from("Connection succeeded."),
                    "objectEncoding" => Number::from(0)
                )
            )
        );
        assert!(handle_message(stream.as_mut()).handle_connect_response(&mut rtmp_context, buffer).await.is_ok());
        assert_eq!(PublisherStatus::Connected, rtmp_context.get_publisher_status().unwrap());

        rtmp_context.set_topic_id(AmfString::new(Uuid::now_v7().to_string()));
        let mut stream = pin!(VecStream::default());
        handle_message(stream.as_mut()).write_release_stream_request(&mut rtmp_context).await.unwrap();
        let mut buffer = ByteBuffer::default();
        buffer.encode(&ReleaseStreamResult);
        assert!(handle_message(stream.as_mut()).handle_release_stream_response(&mut rtmp_context, buffer).await.is_ok());
        assert_eq!(PublisherStatus::Released, rtmp_context.get_publisher_status().unwrap());

        handle_message(stream.as_mut()).write_fc_publish_request(&mut rtmp_context).await.unwrap();
        let mut stream = pin!(VecStream::default());
        let mut buffer = ByteBuffer::default();
        buffer.encode(&OnFcPublish);
        assert!(handle_message(stream.as_mut()).handle_fc_publish_response(&mut rtmp_context, buffer).await.is_ok());
        assert_eq!(PublisherStatus::FcPublished, rtmp_context.get_publisher_status().unwrap());

        handle_message(stream.as_mut()).write_create_stream_request(&mut rtmp_context).await.unwrap();
        let mut stream = pin!(VecStream::default());
        let mut buffer = ByteBuffer::default();
        buffer.encode(&CreateStreamResult::new(0.into()));
        assert!(handle_message(stream.as_mut()).handle_create_stream_response(&mut rtmp_context, buffer).await.is_ok());
        assert_eq!(PublisherStatus::Created, rtmp_context.get_publisher_status().unwrap());

        handle_message(stream.as_mut()).write_publish_request(&mut rtmp_context).await.unwrap();
        let message_id = rtmp_context.get_message_id().unwrap();
        let mut stream = pin!(VecStream::default());
        let mut buffer = ByteBuffer::default();
        buffer.encode(&StreamBegin::new(message_id));
        assert!(handle_message(stream.as_mut()).handle_stream_begin(&mut rtmp_context, buffer).await.is_ok());
        assert_eq!(PublisherStatus::Began, rtmp_context.get_publisher_status().unwrap());

        let topic_id = rtmp_context.get_topic_id().unwrap().clone();
        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &OnStatus::new(
                object!(
                    "level" => AmfString::from("status"),
                    "code" => AmfString::from("NetStream.Publish.Start"),
                    "description" => AmfString::new(format!("{topic_id} is now published")),
                    "details" => topic_id
                )
            )
        );
        assert!(handle_message(stream.as_mut()).handle_publish_response(&mut rtmp_context, buffer).await.is_ok());
        assert_eq!(PublisherStatus::Published, rtmp_context.get_publisher_status().unwrap())
    }

    #[tokio::test]
    async fn ok_subscriber_sequence() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_tc_url("");
        rtmp_context.set_app("");
        rtmp_context.set_client_type(ClientType::Subscriber);

        handle_message(stream.as_mut()).write_connect_request(&mut rtmp_context).await.unwrap();
        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &ConnectResult::new(
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
            )
        );
        assert!(handle_message(stream.as_mut()).handle_connect_response(&mut rtmp_context, buffer).await.is_ok());
        assert_eq!(SubscriberStatus::Connected, rtmp_context.get_subscriber_status().unwrap());

        let mut stream = pin!(VecStream::default());
        handle_message(stream.as_mut()).write_window_acknowledgement_size(&mut rtmp_context).await.unwrap();
        assert_eq!(SubscriberStatus::WindowAcknowledgementSizeGotSent, rtmp_context.get_subscriber_status().unwrap());

        let mut stream = pin!(VecStream::default());
        handle_message(stream.as_mut()).write_create_stream_request(&mut rtmp_context).await.unwrap();
        let mut buffer = ByteBuffer::default();
        buffer.encode(&CreateStreamResult::new(0.into()));
        assert!(handle_message(stream.as_mut()).handle_create_stream_response(&mut rtmp_context, buffer).await.is_ok());
        assert_eq!(SubscriberStatus::Created, rtmp_context.get_subscriber_status().unwrap());

        rtmp_context.set_topic_id(AmfString::new(Uuid::now_v7().to_string()));
        let mut stream = pin!(VecStream::default());
        handle_message(stream.as_mut()).write_fc_subscribe_request(&mut rtmp_context).await.unwrap();
        assert_eq!(SubscriberStatus::FcSubscribed, rtmp_context.get_subscriber_status().unwrap());

        rtmp_context.set_start_time(Some(Duration::default()));
        rtmp_context.set_play_mode(PlayMode::Both);
        let mut stream = pin!(VecStream::default());
        handle_message(stream.as_mut()).write_play_request(&mut rtmp_context).await.unwrap();
        let mut buffer = ByteBuffer::default();
        buffer.encode(&StreamBegin::new(rtmp_context.get_message_id().unwrap()));
        assert!(handle_message(stream.as_mut()).handle_stream_begin(&mut rtmp_context, buffer).await.is_ok());
        assert_eq!(SubscriberStatus::Began, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &OnStatus::new(
                object!(
                    "level" => AmfString::from("status"),
                    "code" => AmfString::from("NetStream.Play.Start"),
                    "description" => AmfString::from("Playing stream")
                )
            )
        );
        assert!(handle_message(stream.as_mut()).handle_play_response(&mut rtmp_context, buffer).await.is_ok());
        assert_eq!(SubscriberStatus::Played, rtmp_context.get_subscriber_status().unwrap());

        rtmp_context.set_buffer_length(30000);
        let mut stream = pin!(VecStream::default());
        handle_message(stream.as_mut()).write_buffer_length(&mut rtmp_context).await.unwrap();
        assert_eq!(SubscriberStatus::BufferLengthGotSent, rtmp_context.get_subscriber_status().unwrap())
    }
}
