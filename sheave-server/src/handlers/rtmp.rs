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
        Handshake,
        Version
    },
    messages::{
        /* Used in common */
        Channel,
        ChunkData,
        CommandError,
        Connect,
        ConnectResult,
        CreateStream,
        CreateStreamResult,
        EventType,
        UserControl,
        OnStatus,
        Audio,
        Video,
        SetDataFrame,
        Acknowledgement,
        amf::v0::{
            AmfString,
            Number,
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
        GetStreamLength,
        GetStreamLengthResult,
        SetPlaylist,
        PlaylistReady,
        Play,
        SetBufferLength,
    },
    net::RtmpReadExt,
    object,
    readers::*,
    writers::*
};
use super::{
    /* Used in common */
    inconsistent_app_path,
    undistinguishable_client,
    empty_topic_id,
    inconsistent_topic_id,
    middlewares::write_acknowledgement,

    /* Publisher-side */
    publish_topic,
    provide_message_id,
    unpublish_topic,
    return_message_id,

    /* Subscriver-side */
    subscribe_topic,
    metadata_not_found,
};

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
                error!("Invalid SHA digest/signature: {:x?}", client_request.get_digest(encryption_algorithm));
                return Err(inconsistent_sha(client_request.get_digest(encryption_algorithm).to_vec()))
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
                error!("Invalid SHA digest/signature: {:x?}", client_response.get_signature());
                return Err(inconsistent_sha(client_response.get_signature().to_vec()))
            } else {
                debug!("Handshake version: {:?}", client_response.get_version());
                debug!("Signature: {:x?}", client_response.get_signature());
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

    async fn handle_window_acknowledgement_size(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let window_acknowledgement_size: WindowAcknowledgementSize = buffer.decode()?;
        rtmp_context.set_window_acknowledgement_size(window_acknowledgement_size);

        /*
         *  NOTE:
         *      Makes status to update during request handling.
         *      Because of response not required.
         */
        rtmp_context.set_subscriber_status(SubscriberStatus::WindowAcknowledgementSizeGotSent);

        info!("Window Acknowledgement Size got handled.");
        Ok(())
    }

    async fn handle_release_stream_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let release_stream_request: ReleaseStream = buffer.decode()?;
        rtmp_context.set_topic_id(release_stream_request.into());

        info!("releaseStream got handled.");
        Ok(())
    }

    async fn handle_fc_publish_request(&mut self, _: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<FcPublish>::decode(&mut buffer)?;

        info!("FCPublish got handled.");
        Ok(())
    }

    async fn handle_create_stream_request(&mut self, _: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<CreateStream>::decode(&mut buffer)?;

        info!("createStream got handled.");
        Ok(())
    }

    async fn handle_fc_subscribe_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let database_url = rtmp_context.get_database_url().unwrap().clone();
        let storage_path = rtmp_context.get_storage_path().unwrap().clone();
        let client_addr = rtmp_context.get_client_addr().unwrap();
        let app = rtmp_context.get_app().unwrap().clone();

        let fc_subscribe_request: FcSubscribe = buffer.decode()?;
        /*
         *  NOTE:
         *      Makes topic to subscribe during request handling.
         *      Because onFCSubscribe command is undefined about its specification.
         */
        let topic = subscribe_topic(&database_url, &storage_path, &app, fc_subscribe_request.get_topic_id(), client_addr).await?;
        rtmp_context.set_topic(topic);
        rtmp_context.set_topic_id(fc_subscribe_request.into());

        rtmp_context.set_subscriber_status(SubscriberStatus::FcSubscribed);

        info!("FCSubscribe got handled.");
        Ok(())
    }

    async fn handle_stream_length_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let get_stream_length_request: GetStreamLength = buffer.decode()?;
        rtmp_context.set_topic_id(get_stream_length_request.into());

        info!("getStreamLength got handled.");
        Ok(())
    }

    async fn handle_playlist_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let set_playlist_request: SetPlaylist = buffer.decode()?;
        rtmp_context.set_playlist(set_playlist_request.into());

        info!("set playlist got handled.");
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

    async fn handle_play_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let play_request: Play = buffer.decode()?;
        let (stream_name, start_time, play_mode) = play_request.into();
        rtmp_context.set_stream_name(stream_name);
        rtmp_context.set_start_time(start_time);
        rtmp_context.set_play_mode(play_mode);

        info!("play got handled.");
        Ok(())
    }

    async fn handle_buffer_length(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let buffer_length: SetBufferLength = buffer.decode()?;
        rtmp_context.set_buffer_length(buffer_length.get_buffering_time());

        /*
         * NOTE:
         *  OBS sends SetBufferLength event also before the play command.
         *  However its step isn't necessarily also other tools have implemented.
         *  Therefore The sheave doesn't count up status except received after the play command implemented as the common step.
         */
        if let Some(SubscriberStatus::Played) = rtmp_context.get_subscriber_status() {
            rtmp_context.set_subscriber_status(SubscriberStatus::BufferLengthGotSent);
        }

        Ok(())
    }

    async fn handle_user_control(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use EventType::*;

        let event_type: EventType = buffer.get_u16_be()?.into();
        match event_type {
            SetBufferLength => self.handle_buffer_length(rtmp_context, buffer).await,
            _ => unimplemented!("Undefined event type: {event_type:?}")
        }
    }

    async fn handle_fc_unpublish_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let database_url = rtmp_context.get_database_url().unwrap().clone();
        let storage_path = rtmp_context.get_storage_path().unwrap().clone();
        let client_addr = rtmp_context.get_client_addr().unwrap();
        let app = rtmp_context.get_app().unwrap().clone();

        let fc_unpublish_request: FcUnpublish = buffer.decode()?;
        unpublish_topic(&database_url, &storage_path, &app, fc_unpublish_request.get_topic_id(), client_addr).await?;
        rtmp_context.reset_topic_id();

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

    async fn handle_publisher_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use PublisherStatus::*;

        let command: AmfString = buffer.decode()?;
        let transaction_id: Number = buffer.decode()?;
        rtmp_context.set_command_name(command.clone());
        rtmp_context.set_transaction_id(transaction_id);

        if command == "FCUnpublish" {
            return self.handle_fc_unpublish_request(rtmp_context, buffer).await
        }
        if command == "deleteStream" {
            return self.handle_delete_stream_request(rtmp_context, buffer).await
        }

        match rtmp_context.get_publisher_status().unwrap() {
            Connected => self.handle_release_stream_request(rtmp_context, buffer).await,
            Released => self.handle_fc_publish_request(rtmp_context, buffer).await,
            FcPublished => self.handle_create_stream_request(rtmp_context, buffer).await,
            Created => self.handle_publish_request(rtmp_context, buffer).await,
            _ => Ok(())
        }
    }

    async fn handle_subscriber_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use SubscriberStatus::*;

        let subscriber_status = rtmp_context.get_subscriber_status().unwrap();

        let command: AmfString = buffer.decode()?;
        let transaction_id: Number = buffer.decode()?;
        rtmp_context.set_command_name(command.clone());
        rtmp_context.set_transaction_id(transaction_id);

        if subscriber_status == FcSubscribed {
            /* NOTE: FFmpeg will send this. */
            if command == "getStreamLength" {
                return self.handle_stream_length_request(rtmp_context, buffer).await
            }
            /* NOTE: OBS will send this. */
            if command == "set_playlist" {
                return self.handle_playlist_request(rtmp_context, buffer).await
            }
        }

        match subscriber_status {
            Connected => Ok(()),
            /* Subscriber sends a Window Acknowledgement Size chunk just after the connect command. */
            WindowAcknowledgementSizeGotSent => self.handle_create_stream_request(rtmp_context, buffer).await,
            Created => self.handle_fc_subscribe_request(rtmp_context, buffer).await,
            AdditionalCommandGotSent => self.handle_play_request(rtmp_context, buffer).await,
            _ => Ok(())
        }
    }

    async fn handle_command_request(&mut self, rtmp_context: &mut RtmpContext, buffer: ByteBuffer) -> IOResult<()> {
        use ClientType::*;

        if let Some(client_type) = rtmp_context.get_client_type() {
            match client_type {
                Publisher => self.handle_publisher_request(rtmp_context, buffer).await,
                Subscriber => self.handle_subscriber_request(rtmp_context, buffer).await
            }
        } else {
            self.handle_connect_request(rtmp_context, buffer).await
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
            // NOTE: Currently @setDataFrame command is used for nothing.
            Decoder::<AmfString>::decode(&mut buffer)?;
        }

        let data: Vec<u8> = buffer.into();
        let flv_tag = FlvTag::new(tag_type, timestamp, data);
        topic.append_flv_tag(flv_tag)?;

        info!("FLV chunk got handled.");
        Ok(())
    }

    async fn write_error_response(&mut self, rtmp_context: &mut RtmpContext, information: Object, error: IOError) -> IOResult<()> {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("_error"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&CommandError::new(information.clone()));
        write_chunk(self.0.as_mut(), rtmp_context, CommandError::CHANNEL.into(), Duration::default(), CommandError::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_information(information);

        error!("{error}");
        return Err(error)
    }

    async fn write_connect_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        use ClientType::*;

        let command_object = rtmp_context.get_command_object().unwrap().clone();

        let client_type = if command_object.get_properties().get("type").is_some() {
            Publisher
        } else if command_object.get_properties().get("fpad").is_some() {
            Subscriber
        } else {
            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetConnection.Connect.UndistinguishableClient"),
                "description" => AmfString::from("Server couldn't distinguish you are either publisher or subscriber.")
            );
            return self.write_error_response(rtmp_context, information, undistinguishable_client()).await
        };

        let app = rtmp_context.get_app().unwrap().clone();
        let requested_app: &AmfString = (&command_object.get_properties()["app"]).into();
        if *requested_app != app {
            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetConnection.Connect.InconsistentAppPath"),
                "description" => AmfString::new(format!("Requested app path is inconsistent. expected: {}, actual: {}", app, requested_app))
            );
            return self.write_error_response(rtmp_context, information, inconsistent_app_path(app, requested_app.clone())).await
        }

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

        rtmp_context.set_client_type(client_type);
        rtmp_context.set_properties(properties);
        rtmp_context.set_information(information);

        match client_type {
            Publisher => rtmp_context.set_publisher_status(PublisherStatus::Connected),
            Subscriber => rtmp_context.set_subscriber_status(SubscriberStatus::Connected)
        }

        info!("connect result got sent.");
        Ok(())
    }

    async fn write_release_stream_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let topic_id = rtmp_context.get_topic_id().unwrap().clone();

        if topic_id.is_empty() {
            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetConnection.ReleaseStream.EmptyTopicPath"),
                "description" => AmfString::from("The topic path must not be empty.")
            );
            return self.write_error_response(rtmp_context, information, empty_topic_id()).await
        }

        let database_url = rtmp_context.get_database_url().unwrap().clone();
        let storage_path = rtmp_context.get_storage_path().unwrap().clone();
        let client_addr = rtmp_context.get_client_addr().unwrap();
        let app = rtmp_context.get_app().unwrap().clone();

        let topic = match publish_topic(&database_url, &storage_path, &app, &topic_id, client_addr).await {
            Ok(topic) => topic,
            Err(e) => {
                let information = object!(
                    "level" => AmfString::from("error"),
                    "code" => AmfString::from("NetConnection.ReleaseStream.StreamIsUnpublished"),
                    "description" => AmfString::new(format!("A stream of {topic_id} is unpublished."))
                );
                return self.write_error_response(rtmp_context, information, e).await
            }
        };

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("_result"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&ReleaseStreamResult);
        write_chunk(self.0.as_mut(), rtmp_context, ReleaseStreamResult::CHANNEL.into(), Duration::default(), ReleaseStreamResult::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_topic(topic);

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
        use ClientType::*;

        let client_type = rtmp_context.get_client_type().unwrap();

        let message_id = provide_message_id();
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("_result"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&CreateStreamResult::new(message_id.into()));
        write_chunk(self.0.as_mut(), rtmp_context, CreateStreamResult::CHANNEL.into(), Duration::default(), CreateStreamResult::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_message_id(message_id);

        match client_type {
            Publisher => rtmp_context.set_publisher_status(PublisherStatus::Created),
            Subscriber => rtmp_context.set_subscriber_status(SubscriberStatus::Created)
        }

        info!("createStream result got sent.");
        Ok(())
    }

    async fn write_stream_length_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let transaction_id = rtmp_context.get_transaction_id();
        let topic = rtmp_context.get_topic_mut().unwrap();

        for result in topic {
            let flv_tag = result?;

            if flv_tag.get_tag_type() == TagType::ScriptData {
                let mut buffer = ByteBuffer::default();
                buffer.put_bytes(flv_tag.get_data());
                let script_data: ScriptDataTag = buffer.decode()?;

                if *script_data.get_name() == "onMetaData" {
                    let duration: &Number = (&script_data.get_value().get_properties()["duration"]).into();
                    let mut buffer = ByteBuffer::default();
                    buffer.encode(&AmfString::from("_result"));
                    buffer.encode(&transaction_id);
                    buffer.encode(&GetStreamLengthResult::new(*duration));
                    write_chunk(self.0.as_mut(), rtmp_context, GetStreamLengthResult::CHANNEL.into(), Duration::default(), GetStreamLengthResult::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

                    rtmp_context.set_subscriber_status(SubscriberStatus::AdditionalCommandGotSent);

                    info!("getStreamLength result got sent.");
                    return Ok(())
                }
            }
        }

        let topic_id = rtmp_context.get_topic_id().unwrap().clone();
        let information = object!(
            "level" => AmfString::from("error"),
            "code" => AmfString::from("NetConnection.GetStreamLength.MetadataNotFound"),
            "description" => AmfString::new(format!("Metadata didn't find in specified topic ID: {topic_id}"))
        );
        self.write_error_response(rtmp_context, information, metadata_not_found(topic_id)).await
    }

    async fn write_playlist_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("playlist_ready"));
        buffer.encode(&PlaylistReady);
        write_chunk(self.0.as_mut(), rtmp_context, PlaylistReady::CHANNEL.into(), Duration::default(), PlaylistReady::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_subscriber_status(SubscriberStatus::AdditionalCommandGotSent);

        info!("playlist_ready got sent.");
        Ok(())
    }

    async fn write_stream_begin(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        use ClientType::*;

        let message_id = rtmp_context.get_message_id().unwrap();
        let mut buffer = ByteBuffer::default();
        buffer.put_u16_be(StreamBegin::EVENT_TYPE.into());
        buffer.encode(&StreamBegin::new(message_id));
        write_chunk(self.0.as_mut(), rtmp_context, StreamBegin::CHANNEL.into(), Duration::default(), StreamBegin::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        match rtmp_context.get_client_type().unwrap() {
            Publisher => rtmp_context.set_publisher_status(PublisherStatus::Began),
            Subscriber => rtmp_context.set_subscriber_status(SubscriberStatus::Began)
        }

        info!("Stream Begin got sent.");
        Ok(())
    }

    async fn write_error_status(&mut self, rtmp_context: &mut RtmpContext, information: Object, error: IOError) -> IOResult<()> {
        let message_id = rtmp_context.get_message_id().unwrap();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("onStatus"));
        buffer.encode(&Number::from(0));
        buffer.encode(&OnStatus::new(information.clone()));
        write_chunk(self.0.as_mut(), rtmp_context, OnStatus::CHANNEL.into(), Duration::default(), OnStatus::MESSAGE_TYPE, message_id, &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_information(information);

        error!("{error}");
        return Err(error)
    }

    async fn write_publish_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let message_id = rtmp_context.get_message_id().unwrap();
        let topic_id = rtmp_context.get_topic_id().unwrap().clone();
        let publishing_name = rtmp_context.get_publishing_name().unwrap().clone();

        if topic_id != publishing_name {
            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetStream.Publish.InconsistentPlaypath"),
                "description" => AmfString::new(format!("Requested name is inconsistent. expected: {topic_id}, actual: {publishing_name}"))
            );
            return self.write_error_status(rtmp_context, information, inconsistent_topic_id(topic_id, publishing_name)).await
        }

        let information = object!(
            "level" => AmfString::from("status"),
            "code" => AmfString::from("NetStream.Publish.Start"),
            "description" => AmfString::new(format!("{publishing_name} is now published")),
            "details" => publishing_name.clone()
        );
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("onStatus"));
        buffer.encode(&Number::from(0));
        buffer.encode(&OnStatus::new(information.clone()));
        write_chunk(self.0.as_mut(), rtmp_context, OnStatus::CHANNEL.into(), Duration::default(), OnStatus::MESSAGE_TYPE, message_id, &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_information(information);

        rtmp_context.set_publisher_status(PublisherStatus::Published);

        info!("onStatus(publish) got sent.");
        Ok(())
    }

    async fn write_play_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let topic_id = rtmp_context.get_topic_id().unwrap().clone();
        let message_id = rtmp_context.get_message_id().unwrap();
        let stream_name = rtmp_context.get_stream_name().unwrap().clone();

        if topic_id != stream_name {
            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetStream.Play.InconsistentTopicPath"),
                "description" => AmfString::new(format!("Requested name is inconsistent. expected: {topic_id}, actual: {stream_name}"))
            );
            return self.write_error_status(rtmp_context, information, inconsistent_topic_id(topic_id, stream_name)).await
        }

        let information = object!(
            "level" => AmfString::from("status"),
            "code" => AmfString::from("NetStream.Play.Start"),
            "description" => AmfString::from("Playing stream.")
        );
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("onStatus"));
        buffer.encode(&Number::from(0));
        buffer.encode(&OnStatus::new(information.clone()));
        write_chunk(self.0.as_mut(), rtmp_context, OnStatus::CHANNEL.into(), Duration::default(), OnStatus::MESSAGE_TYPE, message_id, &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_information(information);

        rtmp_context.set_subscriber_status(SubscriberStatus::Played);

        info!("onStatus(play) got sent.");
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

        info!("FLV data became empty.");
        Err(stream_got_exhausted())
    }
}

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for MessageHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        use MessageType::*;

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
            Acknowledgement => ready!(pin!(self.handle_acknowledgement(rtmp_context, buffer)).poll(cx))?,
            UserControl => ready!(pin!(self.handle_user_control(rtmp_context, buffer)).poll(cx))?,
            WindowAcknowledgementSize => ready!(pin!(self.handle_window_acknowledgement_size(rtmp_context, buffer)).poll(cx))?,
            Audio | Video | Data => {
                let timestamp = rtmp_context.get_last_received_chunk(&chunk_id).unwrap().get_timestamp();
                ready!(pin!(self.handle_flv(rtmp_context, buffer, message_type, timestamp)).poll(cx))?
            },
            Command => ready!(pin!(self.handle_command_request(rtmp_context, buffer)).poll(cx))?,
            other => unimplemented!("Undefined Message: {other:?}")
        }

        if let Some(publisher_status) = rtmp_context.get_publisher_status() {
            match publisher_status {
                PublisherStatus::Connected => pin!(self.write_release_stream_response(rtmp_context)).poll(cx),
                PublisherStatus::Released => pin!(self.write_fc_publish_response(rtmp_context)).poll(cx),
                PublisherStatus::FcPublished => pin!(self.write_create_stream_response(rtmp_context)).poll(cx),
                PublisherStatus::Created => {
                    ready!(pin!(self.write_stream_begin(rtmp_context)).poll(cx))?;
                    pin!(self.write_publish_response(rtmp_context)).poll(cx)
                },
                _ => {
                    /* Just receiving flv after publishing. */
                    Poll::Ready(Ok(()))
                }
            }
        } else if let Some(mut subscriber_status) = rtmp_context.get_subscriber_status() {
            if subscriber_status == SubscriberStatus::FcSubscribed {
                let command = rtmp_context.get_command_name().unwrap().clone();

                if command == "getStreamLength" {
                    return pin!(self.write_stream_length_response(rtmp_context)).poll(cx)
                } else if command == "set_playlist" {
                    return pin!(self.write_playlist_response(rtmp_context)).poll(cx)
                } else {
                    subscriber_status = SubscriberStatus::AdditionalCommandGotSent;
                }
            }

            match subscriber_status {
                SubscriberStatus::WindowAcknowledgementSizeGotSent => pin!(self.write_create_stream_response(rtmp_context)).poll(cx),
                SubscriberStatus::AdditionalCommandGotSent => {
                    ready!(pin!(self.write_stream_begin(rtmp_context)).poll(cx))?;
                    pin!(self.write_play_response(rtmp_context)).poll(cx)
                },
                SubscriberStatus::Played => pin!(self.write_flv(rtmp_context)).poll(cx),
                _ => {
                    /* NOTE: There are plural chunks just to receive. */
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
        let topic_id = rtmp_context.get_topic_id().unwrap().clone();
        rtmp_context.increase_transaction_id();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("FCUnpublish"));
        buffer.encode(&rtmp_context.get_transaction_id());
        buffer.encode(&FcUnpublish::new(topic_id));
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
/// # With publishers
///
/// 1. Checks the application name from the [`Connect`] command.
/// 2. Checks the topic path from the [`ReleaseStream`]/[`FcPublish`] command.
/// 3. Provides a message ID when receives the [`CreateStream`] command.
/// 4. Checks publication informations from the [`Publish`] command.
/// 5. Then receives FLV media data.
///
/// If some error occurs in any step, sends commands which are [`FcUnpublish`] and [`DeleteStream`] to its client, then terminates its connection.
/// These perform to delete the topic path and a message ID from its context.
/// However also these can be sent from clients.
///
/// # With subscribers
///
/// 1. Checks the application name from the [`Connect`] command.
/// 2. Checks partner's bandwidsth from the [`WindowAcknowledgementSize`] message.
/// 3. Provides a message ID when receives the [`CreateStream`] command.
/// 4. Checks the topic path from the [`FcSubscribe`] command.
/// 5. Handles one of additional commands either [`GetStreamLength`] or [`SetPlaylist`].
/// 6. Checkes subscription informaitons from [`Play`] command/
/// 7. The sends FLV media data.
///
/// In Both sides, if receiving data size exceeds server's bandwidth, this reports its thing via the [`Acknowledgement`] message to its client.
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
/// [`WindowAcknowledgementSize`]: sheave_core::messages::WindowAcknowledgementSize
/// [`FcSubscribe`]: sheave_core::messages::FcSubscribe
/// [`GetStreamLength`]: sheave_core::messages::GetStreamLength
/// [`SetPlaylist`]: sheave_core::messages::SetPlaylist
/// [`Play`]: sheave_core::messages::Play
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
    use std::{
        env::temp_dir,
        fs::{
            copy,
            create_dir_all,
            exists,
        },
        net::{
            IpAddr,
            Ipv4Addr,
            SocketAddr,
        },
        path::{
            MAIN_SEPARATOR,
            PathBuf,
        },
        str::FromStr,
    };
    use dotenvy::{
        from_filename,
        var
    };
    use log::LevelFilter;
    use rand::fill;
    use sqlx::{
        Connection,
        MySqlConnection,
        migrate::Migrator,
        query
    };
    use tokio::sync::OnceCell;
    use uuid::Uuid;
    use sheave_core::{
        ecma_array,
        flv::Flv,
        handlers::VecStream,
        handshake::EncryptionAlgorithm,
        messages::{
            ChunkSize,
            SetPlaylist,
            amf::v0::Boolean
        }
    };
    use super::*;

    const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1935);
    const STATEMENT: &str = "INSERT INTO topics (id, client_addr) VALUES (?, ?)";
    static MIGRATOR: OnceCell<()> = OnceCell::const_new();

    async fn migrate_once() {
        let database_url = var("DATABASE_URL")
            .unwrap();
        let mut connection = MySqlConnection::connect(&database_url)
            .await
            .unwrap();
        let migrator = Migrator::new(format!("{}{MAIN_SEPARATOR}migrations", env!("CARGO_MANIFEST_DIR")).as_ref())
            .await
            .unwrap();
        migrator
            .run(&mut connection)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn ok_unsigned_handshake_got_handled() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();

        let sent_encryption_algorithm = EncryptionAlgorithm::NotEncrypted;
        write_encryption_algorithm(stream.as_mut(), sent_encryption_algorithm).await.unwrap();
        let mut sent_client_handshake = Handshake::new(Instant::now().elapsed(), Version::UNSIGNED);
        sent_client_handshake.imprint_digest(sent_encryption_algorithm, Handshake::CLIENT_KEY);
        write_handshake(stream.as_mut(), &sent_client_handshake).await.unwrap();
        let result = handle_handshake(stream.as_mut()).handle_first_handshake(&mut rtmp_context).await;
        assert!(result.is_ok());

        let received_encryption_algorithm = read_encryption_algorithm(stream.as_mut()).await.unwrap();
        let received_server_handshake = read_handshake(stream.as_mut()).await.unwrap();
        let received_client_handshake = read_handshake(stream.as_mut()).await.unwrap();
        assert_eq!(sent_encryption_algorithm, received_encryption_algorithm);
        assert_eq!(sent_client_handshake.get_bytes(), received_client_handshake.get_bytes());

        write_handshake(stream.as_mut(), &received_server_handshake).await.unwrap();
        let result = handle_handshake(stream.as_mut()).handle_second_handshake(&mut rtmp_context).await;
        assert!(result.is_ok());
        let sent_server_handshake = rtmp_context.get_server_handshake().unwrap();
        assert_eq!(received_server_handshake.get_bytes(), sent_server_handshake.get_bytes())
    }

    #[tokio::test]
    async fn err_digest_did_not_match() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();

        let sent_encryption_algorithm = EncryptionAlgorithm::NotEncrypted;
        write_encryption_algorithm(stream.as_mut(), sent_encryption_algorithm).await.unwrap();
        let sent_client_handshake = Handshake::new(Instant::now().elapsed(), Version::LATEST_CLIENT);
        write_handshake(stream.as_mut(), &sent_client_handshake).await.unwrap();
        let result = handle_handshake(stream.as_mut()).handle_first_handshake(&mut rtmp_context).await;
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn err_signature_did_not_match() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();

        let sent_encryption_algorithm = EncryptionAlgorithm::NotEncrypted;
        write_encryption_algorithm(stream.as_mut(), sent_encryption_algorithm).await.unwrap();
        let mut sent_client_handshake = Handshake::new(Instant::now().elapsed(), Version::LATEST_CLIENT);
        sent_client_handshake.imprint_digest(sent_encryption_algorithm, Handshake::CLIENT_KEY);
        write_handshake(stream.as_mut(), &sent_client_handshake).await.unwrap();
        let result = handle_handshake(stream.as_mut()).handle_first_handshake(&mut rtmp_context).await;
        assert!(result.is_ok());

        read_encryption_algorithm(stream.as_mut()).await.unwrap();
        let mut received_server_handshake = read_handshake(stream.as_mut()).await.unwrap();
        read_handshake(stream.as_mut()).await.unwrap();
        let mut invalid_signature_key: [u8; Handshake::CLIENT_KEY.len() + Handshake::COMMON_KEY.len()] = [0; Handshake::CLIENT_KEY.len() + Handshake::COMMON_KEY.len()];
        fill(&mut invalid_signature_key);
        received_server_handshake.imprint_signature(sent_encryption_algorithm, &invalid_signature_key);
        write_handshake(stream.as_mut(), &received_server_handshake).await.unwrap();
        let result = handle_handshake(stream.as_mut()).handle_second_handshake(&mut rtmp_context).await;
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn ok_singed_handshake_got_handled() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();

        let sent_encryption_algorithm = EncryptionAlgorithm::NotEncrypted;
        write_encryption_algorithm(stream.as_mut(), sent_encryption_algorithm).await.unwrap();
        let mut sent_client_handshake = Handshake::new(Instant::now().elapsed(), Version::LATEST_CLIENT);
        sent_client_handshake.imprint_digest(sent_encryption_algorithm, Handshake::CLIENT_KEY);
        write_handshake(stream.as_mut(), &sent_client_handshake).await.unwrap();
        let result = handle_handshake(stream.as_mut()).handle_first_handshake(&mut rtmp_context).await;
        assert!(result.is_ok());

        let received_encryption_algorithm = read_encryption_algorithm(stream.as_mut()).await.unwrap();
        let mut received_server_handshake = read_handshake(stream.as_mut()).await.unwrap();
        let received_client_handshake = read_handshake(stream.as_mut()).await.unwrap();
        let mut server_signature_key: Vec<u8> = Vec::new();
        server_signature_key.extend_from_slice(Handshake::SERVER_KEY);
        server_signature_key.extend_from_slice(Handshake::COMMON_KEY);
        sent_client_handshake.imprint_signature(sent_encryption_algorithm, &server_signature_key);
        assert_eq!(sent_encryption_algorithm, received_encryption_algorithm);
        assert_eq!(sent_client_handshake.get_bytes(), received_client_handshake.get_bytes());

        let mut client_signature_key: Vec<u8> = Vec::new();
        client_signature_key.extend_from_slice(Handshake::CLIENT_KEY);
        client_signature_key.extend_from_slice(Handshake::COMMON_KEY);
        received_server_handshake.imprint_signature(sent_encryption_algorithm, &client_signature_key);
        write_handshake(stream.as_mut(), &received_server_handshake).await.unwrap();
        let result = handle_handshake(stream.as_mut()).handle_second_handshake(&mut rtmp_context).await;
        assert!(result.is_ok());
        let sent_server_handshake = rtmp_context.get_server_handshake().unwrap();
        assert_eq!(received_server_handshake.get_bytes(), sent_server_handshake.get_bytes())
    }

    #[tokio::test]
    async fn ok_signed_handshake_as_ffmpeg_got_handled() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();

        let sent_encryption_algorithm = EncryptionAlgorithm::NotEncrypted;
        write_encryption_algorithm(stream.as_mut(), sent_encryption_algorithm).await.unwrap();
        let mut sent_client_handshake = Handshake::new(Instant::now().elapsed(), Version::LATEST_CLIENT);
        sent_client_handshake.imprint_digest(sent_encryption_algorithm, Handshake::CLIENT_KEY);
        write_handshake(stream.as_mut(), &sent_client_handshake).await.unwrap();
        let result = handle_handshake(stream.as_mut()).handle_first_handshake(&mut rtmp_context).await;
        assert!(result.is_ok());

        let received_encryption_algorithm = read_encryption_algorithm(stream.as_mut()).await.unwrap();
        let received_server_handshake = read_handshake(stream.as_mut()).await.unwrap();
        let received_client_handshake = read_handshake(stream.as_mut()).await.unwrap();
        let mut server_signature_key: Vec<u8> = Vec::new();
        server_signature_key.extend_from_slice(Handshake::SERVER_KEY);
        server_signature_key.extend_from_slice(Handshake::COMMON_KEY);
        sent_client_handshake.imprint_signature(sent_encryption_algorithm, &server_signature_key);
        assert_eq!(sent_encryption_algorithm, received_encryption_algorithm);
        assert_eq!(sent_client_handshake.get_bytes(), received_client_handshake.get_bytes());

        write_handshake(stream.as_mut(), &received_server_handshake).await.unwrap();
        let result = handle_handshake(stream.as_mut()).handle_second_handshake(&mut rtmp_context).await;
        assert!(result.is_ok());
        let sent_server_handshake = rtmp_context.get_server_handshake().unwrap();
        assert_eq!(received_server_handshake.get_bytes(), sent_server_handshake.get_bytes());
    }

    #[tokio::test]
    async fn err_undistinguishable_client() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&Connect::default());
        handle_message(stream.as_mut()).handle_connect_request(&mut rtmp_context, buffer).await.unwrap();
        let result = handle_message(stream.as_mut()).write_connect_response(&mut rtmp_context).await;
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_header = read_message_header(stream.as_mut(), basic_header.get_message_format()).await.unwrap();
        let chunk = read_chunk_data(stream.as_mut(), ChunkSize::default(), message_header.get_message_length().unwrap()).await.unwrap();
        let mut buffer: ByteBuffer = chunk.into();
        let command: AmfString = buffer.decode().unwrap();
        assert!(result.is_err());
        assert_eq!(command, "_error");
        assert!(rtmp_context.get_information().is_some())
    }

    #[tokio::test]
    async fn err_inconsistent_app_path() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_app("ondemand");

        let mut buffer = ByteBuffer::default();
        buffer.encode(&Connect::new(object!("app" => AmfString::default())));
        handle_message(stream.as_mut()).handle_connect_request(&mut rtmp_context, buffer).await.unwrap();
        let result = handle_message(stream.as_mut()).write_connect_response(&mut rtmp_context).await;
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_header = read_message_header(stream.as_mut(), basic_header.get_message_format()).await.unwrap();
        let chunk = read_chunk_data(stream.as_mut(), ChunkSize::default(), message_header.get_message_length().unwrap()).await.unwrap();
        let mut buffer: ByteBuffer = chunk.into();
        let command: AmfString = buffer.decode().unwrap();
        assert!(result.is_err());
        assert_eq!(command, "_error");
        assert!(rtmp_context.get_information().is_some())
    }

    #[tokio::test]
    async fn err_empty_topic_id() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let mut buffer = ByteBuffer::default();
        buffer.encode(&ReleaseStream::new(AmfString::from("")));
        handle_message(stream.as_mut()).handle_release_stream_request(&mut rtmp_context, buffer).await.unwrap();
        let result = handle_message(stream.as_mut()).write_release_stream_response(&mut rtmp_context).await;
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_header = read_message_header(stream.as_mut(), basic_header.get_message_format()).await.unwrap();
        let chunk = read_chunk_data(stream.as_mut(), ChunkSize::default(), message_header.get_message_length().unwrap()).await.unwrap();
        let mut buffer: ByteBuffer = chunk.into();
        let command: AmfString = buffer.decode().unwrap();
        assert!(result.is_err());
        assert_eq!(command, "_error");
        assert!(rtmp_context.get_information().is_some())
    }

    #[tokio::test]
    async fn err_unpublished_stream() {
        if exists(format!("{}{MAIN_SEPARATOR}.env.test", env!("CARGO_MANIFEST_DIR"))).unwrap() {
            from_filename(format!("{}{MAIN_SEPARATOR}.env.test", env!("CARGO_MANIFEST_DIR"))).unwrap();
        }

        MIGRATOR.get_or_init(migrate_once).await;

        let temp_dir = temp_dir();
        let storage_path = format!("{}{MAIN_SEPARATOR}sheave", temp_dir.display());

        let database_url = var("DATABASE_URL").unwrap();

        let app = "ondemand";

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_storage_path(&storage_path);
        rtmp_context.set_database_url(&database_url);
        rtmp_context.set_app(app);
        rtmp_context.set_client_addr(CLIENT_ADDR);

        let topic_id = Uuid::now_v7().to_string();

        let mut buffer = ByteBuffer::default();
        buffer.encode(&ReleaseStream::new(AmfString::new(topic_id)));
        handle_message(stream.as_mut()).handle_release_stream_request(&mut rtmp_context, buffer).await.unwrap();
        let result = handle_message(stream.as_mut()).write_release_stream_response(&mut rtmp_context).await;
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_header = read_message_header(stream.as_mut(), basic_header.get_message_format()).await.unwrap();
        let chunk = read_chunk_data(stream.as_mut(), ChunkSize::default(), message_header.get_message_length().unwrap()).await.unwrap();
        let mut buffer: ByteBuffer = chunk.into();
        let command: AmfString = buffer.decode().unwrap();
        assert!(result.is_err());
        assert_eq!(command, "_error");
        assert!(rtmp_context.get_information().is_some())
    }

    #[tokio::test]
    async fn err_inconsistent_topic_id_in_publish() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_message_id(0);
        rtmp_context.set_topic_id(AmfString::new(Uuid::now_v7().to_string()));

        let mut buffer = ByteBuffer::default();
        buffer.encode(&Publish::new(AmfString::default(), "live".into()));
        handle_message(stream.as_mut()).handle_publish_request(&mut rtmp_context, buffer).await.unwrap();
        let result = handle_message(stream.as_mut()).write_publish_response(&mut rtmp_context).await;
        assert!(result.is_err());
        assert!(rtmp_context.get_information().is_some())
    }

    #[tokio::test]
    async fn err_metadata_not_found() {
        if exists(format!("{}{MAIN_SEPARATOR}.env.test", env!("CARGO_MANIFEST_DIR"))).unwrap() {
            from_filename(format!("{}{MAIN_SEPARATOR}.env.test", env!("CARGO_MANIFEST_DIR"))).unwrap();
        }

        let temp_dir = temp_dir();
        let topic_storage_path = format!("{}{MAIN_SEPARATOR}sheave", temp_dir.display());
        let topic_id = Uuid::now_v7().to_string();
        let topic = {
            create_dir_all(&topic_storage_path).unwrap();
            Flv::create(&format!("{topic_storage_path}{MAIN_SEPARATOR}{topic_id}.flv")).unwrap()
        };

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_topic(topic);

        let mut buffer = ByteBuffer::default();
        buffer.encode(&GetStreamLength::new(AmfString::new(topic_id)));
        handle_message(stream.as_mut()).handle_stream_length_request(&mut rtmp_context, buffer).await.unwrap();
        let result = handle_message(stream.as_mut()).write_stream_length_response(&mut rtmp_context).await;
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_header = read_message_header(stream.as_mut(), basic_header.get_message_format()).await.unwrap();
        let chunk = read_chunk_data(stream.as_mut(), ChunkSize::default(), message_header.get_message_length().unwrap()).await.unwrap();
        let mut buffer: ByteBuffer = chunk.into();
        let command: AmfString = buffer.decode().unwrap();
        assert!(result.is_err());
        assert_eq!(command, "_error");
        assert!(rtmp_context.get_information().is_some())
    }

    #[tokio::test]
    async fn err_inconsistent_topic_id_in_play() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_topic_id(AmfString::new(Uuid::now_v7().to_string()));
        rtmp_context.set_message_id(0);

        let mut buffer = ByteBuffer::default();
        buffer.encode(&Play::new(AmfString::new(Uuid::now_v7().to_string()), Number::from(-2i8)));
        handle_message(stream.as_mut()).handle_play_request(&mut rtmp_context, buffer).await.unwrap();
        let result = handle_message(stream.as_mut()).write_play_response(&mut rtmp_context).await;
        assert!(result.is_err());
        assert!(rtmp_context.get_information().is_some())
    }

    #[tokio::test]
    async fn ok_valid_publisher_sequence() {
        if exists(format!("{}{MAIN_SEPARATOR}.env.test", env!("CARGO_MANIFEST_DIR"))).unwrap() {
            from_filename(format!("{}{MAIN_SEPARATOR}.env.test", env!("CARGO_MANIFEST_DIR"))).unwrap();
        }

        env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::from_str(&var("LOGLEVEL").unwrap_or("error".into())).unwrap())
            .init();

        MIGRATOR.get_or_init(migrate_once).await;

        let temp_dir = temp_dir();
        let storage_path = format!("{}{MAIN_SEPARATOR}sheave", temp_dir.display());

        let database_url = var("DATABASE_URL").unwrap();

        let app = "ondemand";

        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_storage_path(&storage_path);
        rtmp_context.set_database_url(&database_url);
        rtmp_context.set_app(app);
        rtmp_context.set_client_addr(CLIENT_ADDR);

        let topic_id = Uuid::now_v7().to_string();

        query(STATEMENT)
            .bind(&topic_id)
            .bind(&CLIENT_ADDR.to_string())
            .execute(&mut MySqlConnection::connect(&database_url).await.unwrap())
            .await
            .unwrap();

        let mut stream = pin!(VecStream::default());
        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &Connect::new(
                object!(
                    "app" => AmfString::from(app),
                    "type" => AmfString::from("nonprivate")
                )
            )
        );
        handle_message(stream.as_mut()).handle_connect_request(&mut rtmp_context, buffer).await.unwrap();
        assert!(handle_message(stream.as_mut()).write_connect_response(&mut rtmp_context).await.is_ok());
        assert_eq!(PublisherStatus::Connected, rtmp_context.get_publisher_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&ReleaseStream::new(AmfString::new(topic_id.clone())));
        handle_message(stream.as_mut()).handle_release_stream_request(&mut rtmp_context, buffer).await.unwrap();
        let mut stream = pin!(VecStream::default());
        assert!(handle_message(stream.as_mut()).write_release_stream_response(&mut rtmp_context).await.is_ok());
        assert_eq!(PublisherStatus::Released, rtmp_context.get_publisher_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&FcPublish::new(AmfString::new(topic_id.clone())));
        handle_message(stream.as_mut()).handle_fc_publish_request(&mut rtmp_context, buffer).await.unwrap();
        let mut stream = pin!(VecStream::default());
        assert!(handle_message(stream.as_mut()).write_fc_publish_response(&mut rtmp_context).await.is_ok());
        assert_eq!(PublisherStatus::FcPublished, rtmp_context.get_publisher_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&CreateStream);
        handle_message(stream.as_mut()).handle_create_stream_request(&mut rtmp_context, buffer).await.unwrap();
        let mut stream = pin!(VecStream::default());
        assert!(handle_message(stream.as_mut()).write_create_stream_response(&mut rtmp_context).await.is_ok());
        assert_eq!(PublisherStatus::Created, rtmp_context.get_publisher_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&Publish::new(AmfString::new(topic_id), "live".into()));
        handle_message(stream.as_mut()).handle_publish_request(&mut rtmp_context, buffer).await.unwrap();
        let mut stream = pin!(VecStream::default());
        assert!(handle_message(stream.as_mut()).write_stream_begin(&mut rtmp_context).await.is_ok());
        assert_eq!(PublisherStatus::Began, rtmp_context.get_publisher_status().unwrap());
        assert!(handle_message(stream).write_publish_response(&mut rtmp_context).await.is_ok());
        assert_eq!(PublisherStatus::Published, rtmp_context.get_publisher_status().unwrap())
    }

    #[tokio::test]
    async fn ok_valid_subscriber_sequence_in_ffmpeg() {
        if exists(format!("{}{MAIN_SEPARATOR}.env.test", env!("CARGO_MANIFEST_DIR"))).unwrap() {
            from_filename(format!("{}{MAIN_SEPARATOR}.env.test", env!("CARGO_MANIFEST_DIR"))).unwrap();
        }

        MIGRATOR.get_or_init(migrate_once).await;

        let temp_dir = temp_dir();
        let storage_path = format!("{}{MAIN_SEPARATOR}sheave", temp_dir.display());
        let app = "ondemand";
        let copy_to = format!("{storage_path}{MAIN_SEPARATOR}{app}");
        create_dir_all(&copy_to).unwrap();

        let mut resources_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        resources_path.pop();
        resources_path.push("resources");
        resources_path.push("test.flv");
        let topic_id = Uuid::now_v7().to_string();
        copy(resources_path, format!("{copy_to}{MAIN_SEPARATOR}{topic_id}.flv")).unwrap();

        let database_url = var("DATABASE_URL").unwrap();

        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_storage_path(&storage_path);
        rtmp_context.set_database_url(&database_url);
        rtmp_context.set_app(app);
        rtmp_context.set_client_addr(CLIENT_ADDR);

        query(STATEMENT)
            .bind(&topic_id)
            .bind(&CLIENT_ADDR.to_string())
            .execute(&mut MySqlConnection::connect(&database_url).await.unwrap())
            .await
            .unwrap();

        let mut stream = pin!(VecStream::default());
        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &Connect::new(
                object!(
                    "app" => AmfString::from(app),
                    "fpad" => Boolean::new(0)
                )
            )
        );
        handle_message(stream.as_mut()).handle_connect_request(&mut rtmp_context, buffer).await.unwrap();
        assert!(handle_message(stream.as_mut()).write_connect_response(&mut rtmp_context).await.is_ok());
        assert_eq!(SubscriberStatus::Connected, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&WindowAcknowledgementSize::default());
        handle_message(stream.as_mut()).handle_window_acknowledgement_size(&mut rtmp_context, buffer).await.unwrap();
        assert_eq!(SubscriberStatus::WindowAcknowledgementSizeGotSent, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&CreateStream);
        handle_message(stream.as_mut()).handle_create_stream_request(&mut rtmp_context, buffer).await.unwrap();
        let mut stream = pin!(VecStream::default());
        assert!(handle_message(stream.as_mut()).write_create_stream_response(&mut rtmp_context).await.is_ok());
        assert_eq!(SubscriberStatus::Created, rtmp_context.get_subscriber_status().unwrap());

        let mut stream = pin!(VecStream::default());
        let mut buffer = ByteBuffer::default();
        buffer.encode(&FcSubscribe::new(AmfString::new(topic_id.clone())));
        handle_message(stream.as_mut()).handle_fc_subscribe_request(&mut rtmp_context, buffer).await.unwrap();
        assert_eq!(SubscriberStatus::FcSubscribed, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&GetStreamLength::new(AmfString::new(topic_id.clone())));
        handle_message(stream.as_mut()).handle_stream_length_request(&mut rtmp_context, buffer).await.unwrap();
        let mut stream = pin!(VecStream::default());
        assert!(handle_message(stream.as_mut()).write_stream_length_response(&mut rtmp_context).await.is_ok());
        assert_eq!(SubscriberStatus::AdditionalCommandGotSent, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &Play::new(
                AmfString::new(topic_id),
                Number::from(-2i8)
            )
        );
        handle_message(stream.as_mut()).handle_play_request(&mut rtmp_context, buffer).await.unwrap();
        let mut stream = pin!(VecStream::default());
        assert!(handle_message(stream.as_mut()).write_stream_begin(&mut rtmp_context).await.is_ok());
        assert_eq!(SubscriberStatus::Began, rtmp_context.get_subscriber_status().unwrap());
        assert!(handle_message(stream.as_mut()).write_play_response(&mut rtmp_context).await.is_ok());
        assert_eq!(SubscriberStatus::Played, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&SetBufferLength::new(rtmp_context.get_message_id().unwrap(), 0));
        handle_message(stream.as_mut()).handle_buffer_length(&mut rtmp_context, buffer).await.unwrap();
        assert_eq!(SubscriberStatus::BufferLengthGotSent, rtmp_context.get_subscriber_status().unwrap())
    }

    #[tokio::test]
    async fn ok_valid_subscriber_sequence_in_obs() {
        if exists(format!("{}{MAIN_SEPARATOR}.env.test", env!("CARGO_MANIFEST_DIR"))).unwrap() {
            from_filename(format!("{}{MAIN_SEPARATOR}.env.test", env!("CARGO_MANIFEST_DIR"))).unwrap();
        }

        MIGRATOR.get_or_init(migrate_once).await;

        let temp_dir = temp_dir();
        let storage_path = format!("{}{MAIN_SEPARATOR}sheave", temp_dir.display());

        let app = "ondemand";
        let copy_to = format!("{storage_path}{MAIN_SEPARATOR}{app}");
        create_dir_all(&copy_to).unwrap();

        let mut resources_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        resources_path.pop();
        resources_path.push("resources");
        resources_path.push("test.flv");
        let topic_id = Uuid::now_v7().to_string();
        copy(resources_path, &format!("{copy_to}{MAIN_SEPARATOR}{topic_id}.flv")).unwrap();

        let database_url = var("DATABASE_URL").unwrap();

        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_storage_path(&storage_path);
        rtmp_context.set_database_url(&database_url);
        rtmp_context.set_app(app);
        rtmp_context.set_client_addr(CLIENT_ADDR);

        query(STATEMENT)
            .bind(&topic_id)
            .bind(&CLIENT_ADDR.to_string())
            .execute(&mut MySqlConnection::connect(&database_url).await.unwrap())
            .await
            .unwrap();

        let mut stream = pin!(VecStream::default());
        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &Connect::new(
                object!(
                    "app" => AmfString::from(app),
                    "fpad" => Boolean::new(0)
                )
            )
        );
        handle_message(stream.as_mut()).handle_connect_request(&mut rtmp_context, buffer).await.unwrap();
        assert!(handle_message(stream.as_mut()).write_connect_response(&mut rtmp_context).await.is_ok());
        assert_eq!(SubscriberStatus::Connected, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&WindowAcknowledgementSize::default());
        handle_message(stream.as_mut()).handle_window_acknowledgement_size(&mut rtmp_context, buffer).await.unwrap();
        assert_eq!(SubscriberStatus::WindowAcknowledgementSizeGotSent, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&CreateStream);
        handle_message(stream.as_mut()).handle_create_stream_request(&mut rtmp_context, buffer).await.unwrap();
        let mut stream = pin!(VecStream::default());
        assert!(handle_message(stream.as_mut()).write_create_stream_response(&mut rtmp_context).await.is_ok());
        assert_eq!(SubscriberStatus::Created, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&FcSubscribe::new(AmfString::new(topic_id.clone())));
        handle_message(stream.as_mut()).handle_fc_subscribe_request(&mut rtmp_context, buffer).await.unwrap();
        assert_eq!(SubscriberStatus::FcSubscribed, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &SetPlaylist::new(
                ecma_array!(
                    "0" => AmfString::new(topic_id.clone())
                )
            )
        );
        handle_message(stream.as_mut()).handle_playlist_request(&mut rtmp_context, buffer).await.unwrap();
        let mut stream = pin!(VecStream::default());
        assert!(handle_message(stream.as_mut()).write_playlist_response(&mut rtmp_context).await.is_ok());
        assert_eq!(SubscriberStatus::AdditionalCommandGotSent, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &Play::new(
                AmfString::new(topic_id),
                Number::from(-2i8)
            )
        );
        handle_message(stream.as_mut()).handle_play_request(&mut rtmp_context, buffer).await.unwrap();
        let mut stream = pin!(VecStream::default());
        assert!(handle_message(stream.as_mut()).write_stream_begin(&mut rtmp_context).await.is_ok());
        assert_eq!(SubscriberStatus::Began, rtmp_context.get_subscriber_status().unwrap());
        assert!(handle_message(stream.as_mut()).write_play_response(&mut rtmp_context).await.is_ok());
        assert_eq!(SubscriberStatus::Played, rtmp_context.get_subscriber_status().unwrap());

        let mut buffer = ByteBuffer::default();
        buffer.encode(&SetBufferLength::new(rtmp_context.get_message_id().unwrap(), 0));
        handle_message(stream.as_mut()).handle_buffer_length(&mut rtmp_context, buffer).await.unwrap();
        assert_eq!(SubscriberStatus::BufferLengthGotSent, rtmp_context.get_subscriber_status().unwrap())
    }
}
