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
    flv::{
        Flv,
        tags::*
    },
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
    undistinguishable_client,
    empty_playpath,
    inconsistent_playpath,
    middlewares::write_acknowledgement,

    /* Publisher-side */
    publish_topic,
    provide_message_id,
    unpublish_topic,
    return_message_id,

    /* Subscriver-side */
    subscribe_topic,
    did_get_published,
    metadata_not_found,
    stream_is_unpublished
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
                error!("Client side digest is inconsistent!\nencryption_algorithm: {:?}\ndigest: {:x?}", encryption_algorithm, client_request.get_digest(encryption_algorithm));
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
        rtmp_context.set_playpath(release_stream_request.into());

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
        let topic_storage_url = rtmp_context.get_topic_storage_url().unwrap();

        let fc_subscribe_request: FcSubscribe = buffer.decode()?;
        /*
         *  NOTE:
         *      Makes topic to subscribe during request handling.
         *      Because onFCSubscribe command is undefined about its specification.
         */
        let topic = subscribe_topic(topic_storage_url, fc_subscribe_request.get_subscribepath()).await?;
        rtmp_context.set_topic(topic);
        rtmp_context.set_subscribepath(fc_subscribe_request.into());

        rtmp_context.set_subscriber_status(SubscriberStatus::FcSubscribed);

        info!("FCSubscribe got handled.");
        Ok(())
    }

    async fn handle_get_stream_length_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let get_stream_length_request: GetStreamLength = buffer.decode()?;
        rtmp_context.set_playpath(get_stream_length_request.into());

        info!("getStreamLength got handled.");
        Ok(())
    }

    async fn handle_set_playlist_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let set_playlist_request: SetPlaylist = buffer.decode()?;
        rtmp_context.set_playlist(set_playlist_request.into());

        info!("set playlist got handled.");
        Ok(())
    }

    async fn handle_additional_command_request(&mut self, rtmp_context: &mut RtmpContext, buffer: ByteBuffer) -> IOResult<()> {
        let additional_command = rtmp_context.get_command_name().unwrap();
        /* NOTE: FFmepg will send this. */
        if *additional_command == "getStreamLength" {
            self.handle_get_stream_length_request(rtmp_context, buffer).await
        }
        /* NOTE: OBS will send this. */
        else if *additional_command == "set_playlist" {
            self.handle_set_playlist_request(rtmp_context, buffer).await
        } else {
            Ok(())
        }
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

    async fn handle_set_buffer_length(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        Decoder::<SetBufferLength>::decode(&mut buffer)?;

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
            SetBufferLength => self.handle_set_buffer_length(rtmp_context, buffer).await,
            _ => unimplemented!("Undefined event type: {event_type:?}")
        }
    }

    async fn handle_fc_unpublish_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        let topic_storage_url = rtmp_context.get_topic_storage_url().unwrap().clone();
        let client_addr = rtmp_context.get_client_addr().unwrap();
        let fc_unpublish_request: FcUnpublish = buffer.decode()?;
        unpublish_topic(&topic_storage_url, fc_unpublish_request.get_playpath(), client_addr).await?;

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

    async fn handle_publisher_request(&mut self, rtmp_context: &mut RtmpContext, mut buffer: ByteBuffer) -> IOResult<()> {
        use PublisherStatus::*;

        let command: AmfString = buffer.decode()?;
        let transaction_id: Number = buffer.decode()?;

        if command == "FCUnpublish" {
            return self.handle_fc_unpublish_request(rtmp_context, buffer).await
        }
        if command == "deleteStream" {
            return self.handle_delete_stream_request(rtmp_context, buffer).await
        }

        rtmp_context.set_command_name(command);
        rtmp_context.set_transaction_id(transaction_id);

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

        let command: AmfString = buffer.decode()?;
        let transaction_id: Number = buffer.decode()?;
        rtmp_context.set_command_name(command);
        rtmp_context.set_transaction_id(transaction_id);

        match rtmp_context.get_subscriber_status().unwrap() {
            Connected => Ok(()),
            /* Subscriber sends a Window Acknowledgement Size chunk just after the connect command. */
            WindowAcknowledgementSizeGotSent => self.handle_create_stream_request(rtmp_context, buffer).await,
            Created => self.handle_fc_subscribe_request(rtmp_context, buffer).await,
            FcSubscribed => self.handle_additional_command_request(rtmp_context, buffer).await,
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
        let input = rtmp_context.get_topic().unwrap();

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

        let command_object = rtmp_context.get_command_object().unwrap().get_properties();
        let client_type = if command_object.get("type").is_some() {
            Publisher
        } else if command_object.get("fpad").is_some() {
            Subscriber
        } else {
            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetConnection.Connect.UndistinguishableClient"),
                "description" => AmfString::from("Server couldn't distinguish you are either publisher or subscriber.")
            );
            return self.write_error_response(rtmp_context, information, undistinguishable_client()).await
        };

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

        match client_type {
            Publisher => rtmp_context.set_publisher_status(PublisherStatus::Connected),
            Subscriber => rtmp_context.set_subscriber_status(SubscriberStatus::Connected)
        }

        info!("connect result got sent.");
        Ok(())
    }

    async fn write_release_stream_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let playpath = rtmp_context.get_playpath().unwrap();
        if playpath.is_empty() {
            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetConnection.ReleaseStream.EmptyPlaypath"),
                "description" => AmfString::from("Playpath must not be empty.")
            );
            return self.write_error_response(rtmp_context, information, empty_playpath()).await
        }

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
        let client_addr = rtmp_context.get_client_addr().unwrap();
        let playpath = rtmp_context.get_playpath().unwrap().clone();
        let topic_storage_url = rtmp_context.get_topic_storage_url().unwrap().clone();
        publish_topic(&topic_storage_url, &playpath, client_addr).await?;
        let topic = Flv::create(&playpath)?;
        rtmp_context.set_topic(topic);

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

    async fn write_get_stream_length_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let topic_storage_url = rtmp_context.get_topic_storage_url().unwrap().clone();
        let client_addr = rtmp_context.get_client_addr().unwrap();
        let playpath = rtmp_context.get_playpath().unwrap().clone();
        let transaction_id = rtmp_context.get_transaction_id();

        if did_get_published(&topic_storage_url, &playpath, client_addr).await {
            let topic = subscribe_topic(&topic_storage_url, &playpath).await?;
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

            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetConnection.GetStreamLength.MetadataNotFound"),
                "description" => AmfString::from("Metadata didn't find in specified playpath: {playpath}")
            );
            self.write_error_response(rtmp_context, information, metadata_not_found(playpath)).await
        } else {
            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetConnection.GetStreamLength.Unpublished"),
                "description" => AmfString::from("Specified playpath is unpublished.")
            );
            self.write_error_response(rtmp_context, information, stream_is_unpublished(playpath)).await
        }
    }

    async fn write_set_playlist_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("playlist_ready"));
        buffer.encode(&PlaylistReady);
        write_chunk(self.0.as_mut(), rtmp_context, PlaylistReady::CHANNEL.into(), Duration::default(), PlaylistReady::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer)).await?;

        rtmp_context.set_subscriber_status(SubscriberStatus::AdditionalCommandGotSent);

        info!("playlist_ready got sent.");
        Ok(())
    }

    async fn write_additional_command_response(&mut self, rtmp_context: &mut RtmpContext) -> IOResult<()> {
        let additional_command = rtmp_context.get_command_name().unwrap();
        /* NOTE: FFmpeg will require of this. */
        if *additional_command == "getStreamLength" {
            self.write_get_stream_length_response(rtmp_context).await
        }
        /* NOTE: OBS will require of this. */
        else if *additional_command == "set_playlist" {
            self.write_set_playlist_response(rtmp_context).await
        } else {
            /*
             * NOTE:
             *  This step won't respond any error even if its command is neither getStreamLength nor set_playlist.
             *  Because this mayn't be requested.
             */
            rtmp_context.set_subscriber_status(SubscriberStatus::AdditionalCommandGotSent);
            Ok(())
        }
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
        let playpath = rtmp_context.get_playpath().unwrap().clone();
        let publishing_name = rtmp_context.get_publishing_name().unwrap().clone();

        if playpath != publishing_name {
            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetStream.Publish.InconsistentPlaypath"),
                "description" => AmfString::new(format!("Requested name is inconsistent. expected: {playpath}, actual: {publishing_name}"))
            );
            return self.write_error_status(rtmp_context, information, inconsistent_playpath(playpath, publishing_name)).await
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
        let message_id = rtmp_context.get_message_id().unwrap();
        let subscribepath = rtmp_context.get_subscribepath().unwrap().clone();
        let stream_name = rtmp_context.get_stream_name().unwrap().clone();

        if subscribepath != stream_name {
            let information = object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetStream.Play.InconsistentPlaypath"),
                "description" => AmfString::from("Requested name is inconsistent.")
            );
            return self.write_error_status(rtmp_context, information, inconsistent_playpath(subscribepath, stream_name)).await
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
        } else if let Some(subscriber_status) = rtmp_context.get_subscriber_status() {
            match subscriber_status {
                SubscriberStatus::WindowAcknowledgementSizeGotSent => pin!(self.write_create_stream_response(rtmp_context)).poll(cx),
                SubscriberStatus::FcSubscribed => pin!(self.write_additional_command_response(rtmp_context)).poll(cx),
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
/// # With publishers
///
/// 1. Checks the application name from the [`Connect`] command.
/// 2. Checks the playpath from the [`ReleaseStream`]/[`FcPublish`] command.
/// 3. Provides a message ID when receives the [`CreateStream`] command.
/// 4. Checks publication informations from the [`Publish`] command.
/// 5. Then receives FLV media data.
///
/// If some error occurs in any step, sends commands which are [`FcUnpublish`] and [`DeleteStream`] to its client, then terminates its connection.
/// These perform to delete the playpath and a message ID from its context.
/// However also these can be sent from clients.
///
/// # With subscribers
///
/// 1. Checks the application name from the [`Connect`] command.
/// 2. Checks partner's bandwidsth from the [`WindowAcknowledgementSize`] message.
/// 3. Provides a message ID when receives the [`CreateStream`] command.
/// 4. Checks the subscribepath from the [`FcSubscribe`] command.
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
