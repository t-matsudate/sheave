mod last_chunk;

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::Duration
};
use crate::{
    handshake::{
        EncryptionAlgorithm,
        Handshake
    },
    messages::{
        ChunkSize,
        WindowAcknowledgementSize,
        PeerBandwidth,
        PlayMode,
        amf::v0::{
            Number,
            AmfString,
            Object,
            EcmaArray
        }
    },
    flv::Flv
};
use super::{
    ClientType,
    PublisherStatus,
    SubscriberStatus
};
pub use self::last_chunk::*;

/// RTMP's common contexts.
///
/// Many fields are optional by default.
/// Because these data need for both of client and server but are sent/received later.
/// Therefore the `default` constructor has been prepared instead of such as `new`.
///
/// # Examples
///
/// ```rust
/// use sheave_core::handlers::RtmpContext;
///
/// // When you create this struct.
/// RtmpContext::default();
/// ```
#[derive(Debug)]
pub struct RtmpContext {
    signed: bool,
    receiving_chunk_size: ChunkSize,
    sending_chunk_size: ChunkSize,
    window_acknowledgement_size: WindowAcknowledgementSize,
    peer_bandwidth: PeerBandwidth,
    buffer_length: u32,
    last_transaction_id: Number,
    database_url: Option<String>,
    storage_path: Option<String>,
    client_addr: Option<SocketAddr>,
    app: Option<AmfString>,
    topic_id: Option<AmfString>,
    tc_url: Option<AmfString>,
    last_command_name: Option<AmfString>,
    client_type: Option<ClientType>,
    publisher_status: Option<PublisherStatus>,
    subscriber_status: Option<SubscriberStatus>,
    encryption_algorithm: Option<EncryptionAlgorithm>,
    client_handshake: Option<Handshake>,
    server_handshake: Option<Handshake>,
    command_object: Option<Object>,
    properties: Option<Object>,
    information: Option<Object>,
    message_id: Option<u32>,
    playlist: Option<EcmaArray>,
    publishing_name: Option<AmfString>,
    publishing_type: Option<AmfString>,
    stream_name: Option<AmfString>,
    start_time: Option<Duration>,
    play_mode: Option<PlayMode>,
    await_duration: Option<Duration>,
    topic: Option<Flv>,
    last_received_chunks: HashMap<u16, LastChunk>,
    last_sent_chunks: HashMap<u16, LastChunk>
}

impl Default for RtmpContext {
    fn default() -> Self {
        Self {
            signed: bool::default(),
            receiving_chunk_size: ChunkSize::default(),
            sending_chunk_size: ChunkSize::default(),
            window_acknowledgement_size: WindowAcknowledgementSize::default(),
            peer_bandwidth: PeerBandwidth::default(),
            buffer_length: 30000,
            last_transaction_id: Number::default(),
            database_url: Option::default(),
            storage_path: Option::default(),
            client_addr: Option::default(),
            app: Option::default(),
            topic_id: Option::default(),
            tc_url: Option::default(),
            last_command_name: Option::default(),
            client_type: Option::default(),
            publisher_status: Option::default(),
            subscriber_status: Option::default(),
            encryption_algorithm: Option::default(),
            client_handshake: Option::default(),
            server_handshake: Option::default(),
            command_object: Option::default(),
            properties: Option::default(),
            information: Option::default(),
            message_id: Option::default(),
            playlist: Option::default(),
            publishing_name: Option::default(),
            publishing_type: Option::default(),
            stream_name: Option::default(),
            start_time: Option::default(),
            play_mode: Option::default(),
            await_duration: Option::default(),
            topic: Option::default(),
            last_received_chunks: HashMap::default(),
            last_sent_chunks: HashMap::default()
        }
    }
}

impl RtmpContext {
    /// Gets a mutable reference via this wrapped by `Arc`.
    ///
    /// Sheave uses this after wrapping into `Arc`.
    /// Because of making this shareable between every handling steps.
    pub fn make_weak_mut<'a>(self: &'a Arc<Self>) -> &'a mut Self {
        unsafe { &mut *(Arc::downgrade(self).as_ptr() as *mut Self) }
    }

    /// Stores a flag to mean this handshake is signed.
    pub fn set_signed(&mut self, signed: bool) {
        self.signed = signed;
    }

    /// Indicates whether the handshake is signed.
    pub fn is_signed(&mut self) -> bool {
        self.signed
    }

    /// Sets a chunking size which reads from streams.
    pub fn set_receiving_chunk_size(&mut self, chunk_size: ChunkSize) {
        self.receiving_chunk_size = chunk_size;
    }

    /// Gets a chunking size which reads from streams.
    pub fn get_receiving_chunk_size(&mut self) -> ChunkSize {
        self.receiving_chunk_size
    }

    /// Sets a chunking size which writes into streams.
    pub fn set_sending_chunk_size(&mut self, chunk_size: ChunkSize) {
        self.sending_chunk_size = chunk_size;
    }

    /// Gets a chunkign size which writes into stream.
    pub fn get_sending_chunk_size(&mut self) -> ChunkSize {
        self.sending_chunk_size
    }

    /// Sets the window acknowledgement size.
    pub fn set_window_acknowledgement_size(&mut self, window_acknowledgement_size: WindowAcknowledgementSize) {
        self.window_acknowledgement_size = window_acknowledgement_size;
    }

    /// Gets the window acknowledgement size.
    pub fn get_window_acknowledgement_size(&mut self) -> WindowAcknowledgementSize {
        self.window_acknowledgement_size
    }

    /// Sets the peer bandwidth.
    pub fn set_peer_bandwidth(&mut self, peer_bandwidth: PeerBandwidth) {
        self.peer_bandwidth = peer_bandwidth;
    }

    /// Gets the peer bandwidth.
    pub fn get_peer_bandwidth(&mut self) -> PeerBandwidth {
        self.peer_bandwidth
    }

    /// Sets the buffer length.
    pub fn set_buffer_length(&mut self, buffer_length: u32) {
        self.buffer_length = buffer_length;
    }

    /// Gets the buffer length.
    pub fn get_buffer_length(&mut self) -> u32 {
        self.buffer_length
    }

    /// Sets a transaction ID.
    ///
    /// Mainly, this is used by server side contexts.
    /// Because of servers should echo same transaction ID in its response.
    pub fn set_transaction_id(&mut self, transaction_id: Number) {
        self.last_transaction_id = transaction_id;
    }

    /// Gets a transaction ID sent.
    pub fn get_transaction_id(&mut self) -> Number {
        self.last_transaction_id
    }

    /// Increases current transaction ID.
    ///
    /// Mainly, this is used by client side contexts.
    /// Because of clients should count which transaction is it now on.
    pub fn increase_transaction_id(&mut self) {
        self.last_transaction_id += 1f64;
    }

    /// Sets the database url.
    pub fn set_database_url(&mut self, database_url: &str) {
        self.database_url = Some(database_url.into());
    }

    /// Gets the database url.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_database_url().is_none())
    /// ```
    pub fn get_database_url(&mut self) -> Option<&String> {
        self.database_url.as_ref()
    }

    /// Sets the storage path.
    pub fn set_storage_path(&mut self, storage_path: &str) {
        self.storage_path = Some(storage_path.into());
    }

    /// Gets the storage path.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_storage_path().is_none())
    /// ```
    pub fn get_storage_path(&mut self) -> Option<&String> {
        self.storage_path.as_ref()
    }

    /// Sets a client IP address.
    pub fn set_client_addr(&mut self, client_addr: SocketAddr) {
        self.client_addr = Some(client_addr);
    }

    /// Gets a client IP address.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_client_addr().is_none())
    /// ```
    pub fn get_client_addr(&mut self) -> Option<SocketAddr> {
        self.client_addr
    }

    /// Sets the `app` name.
    ///
    /// This can be contained in a request URI of RTMP.
    pub fn set_app(&mut self, app: &str) {
        self.app = Some(AmfString::from(app));
    }

    /// Gets the `app` name.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_app().is_none())
    /// ```
    pub fn get_app(&mut self) -> Option<&AmfString> {
        self.app.as_ref()
    }

    /// Sets a `topic_id` (e.g. filename) sent from a client.
    pub fn set_topic_id(&mut self, topic_id: AmfString) {
        self.topic_id = Some(topic_id);
    }

    /// Resets a `topic_id` from this context.
    ///
    /// This is prepared for deleting a `topic_id` when receives the `FCUnpublish` command.
    pub fn reset_topic_id(&mut self) {
        self.topic_id = None;
    }

    /// Gets a `topic_id` (e.g. filename) sent from a client.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_topic_id().is_none())
    /// ```
    pub fn get_topic_id(&mut self) -> Option<&AmfString> {
        self.topic_id.as_ref()
    }

    /// Sets the `tcUrl`. This is a full URL in the RTMP request like following form.
    ///
    /// `rtmp://hostname/[app]/[topic_path]`
    ///
    /// Where `app` and `topic_path` can be unspecified.
    /// Clients specify above URL at the start of RTMP requests.
    /// Then the server checks `app` and `topic_path` in client-side `Connect` commands (if they are specified).
    pub fn set_tc_url(&mut self, tc_url: &str) {
        self.tc_url = Some(AmfString::from(tc_url));
    }

    /// Gets the `tcUrl`.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_tc_url().is_none())
    /// ```
    pub fn get_tc_url(&mut self) -> Option<&AmfString> {
        self.tc_url.as_ref()
    }

    /// Sets last command name.
    pub fn set_command_name(&mut self, command_name: AmfString) {
        self.last_command_name = Some(command_name);
    }

    /// Gets last command name.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_command_name().is_none())
    /// ```
    pub fn get_command_name(&mut self) -> Option<&AmfString> {
        self.last_command_name.as_ref()
    }

    /// Sets that its client is either publisher or subscriber.
    ///
    /// Curently, the server distinguishes this by referring specific field in a command object which a connect command has.
    /// e.g. "fpad", "capabilities" etc.
    pub fn set_client_type(&mut self, client_type: ClientType) {
        self.client_type = Some(client_type);
    }

    /// Gets the type that its client is which either publisher or subscriber.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_client_type().is_none())
    /// ```
    pub fn get_client_type(&mut self) -> Option<ClientType> {
        self.client_type
    }

    /// Sets one of publisher's status.
    pub fn set_publisher_status(&mut self, publisher_status: PublisherStatus) {
        self.publisher_status = Some(publisher_status);
    }

    /// Gets one of publisher's status.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_publisher_status().is_none())
    /// ```
    pub fn get_publisher_status(&mut self) -> Option<PublisherStatus> {
        self.publisher_status
    }

    /// Sets one of subscriber's status.
    pub fn set_subscriber_status(&mut self, subscriber_status: SubscriberStatus) {
        self.subscriber_status = Some(subscriber_status);
    }

    /// Gets one of subscriber's status.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_subscriber_status().is_none())
    /// ```
    pub fn get_subscriber_status(&mut self) -> Option<SubscriberStatus> {
        self.subscriber_status
    }

    /// Stores the algorithm to encrypt this handshake.
    pub fn set_encryption_algorithm(&mut self, encryption_algorithm: EncryptionAlgorithm) {
        self.encryption_algorithm = Some(encryption_algorithm);
    }

    /// Gets specieifed algorithm to encrypt this handshake.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_encryption_algorithm().is_none())
    /// ```
    pub fn get_encryption_algorithm(&mut self) -> Option<EncryptionAlgorithm> {
        self.encryption_algorithm
    }

    /// Stores a cleint-side handshake bytes.
    pub fn set_client_handshake(&mut self, handshake: Handshake) {
        self.client_handshake = Some(handshake);
    }

    /// Gets a client-side handshake bytes.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_client_handshake().is_none())
    /// ```
    pub fn get_client_handshake(&mut self) -> Option<&Handshake> {
        self.client_handshake.as_ref()
    }

    /// Gets a client-side handshake bytes as mutable.
    ///
    /// Note:
    ///
    /// * This is currently used for only testing (also intagration tests contained).
    /// * This can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_client_handshake_mut().is_none())
    /// ```
    pub fn get_client_handshake_mut(&mut self) -> Option<&mut Handshake> {
        self.client_handshake.as_mut()
    }

    /// Stores a server-side handshake bytes.
    pub fn set_server_handshake(&mut self, handshake: Handshake) {
        self.server_handshake = Some(handshake);
    }

    /// Gets a server-side handshake bytes.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_server_handshake().is_none())
    /// ```
    pub fn get_server_handshake(&mut self) -> Option<&Handshake> {
        self.server_handshake.as_ref()
    }

    /// Sets a command object sent from a client.
    pub fn set_command_object(&mut self, command_object: Object) {
        self.command_object = Some(command_object);
    }

    /// Gets a command object sent from a client.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_command_object().is_none())
    /// ```
    pub fn get_command_object(&mut self) -> Option<&Object> {
        self.command_object.as_ref()
    }

    /// Sets a properties object of a server.
    pub fn set_properties(&mut self, properties: Object) {
        self.properties = Some(properties);
    }

    /// Gets a properties object of a server.
    ///
    /// Note this can return `None`. e.g. When this field is dafault as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_properties().is_none())
    /// ```
    pub fn get_properties(&mut self) -> Option<&Object> {
        self.properties.as_ref()
    }

    /// Sets a information object of a server.
    pub fn set_information(&mut self, information: Object) {
        self.information = Some(information);
    }

    /// Gets a information object of a server.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_information().is_none())
    /// ```
    pub fn get_information(&mut self) -> Option<&Object> {
        self.information.as_ref()
    }

    /// Sets a message ID of this stream.
    pub fn set_message_id(&mut self, message_id: u32) {
        self.message_id = Some(message_id);
    }

    /// Resets a message ID from this context.
    ///
    /// This is prepared for deleting tne `message_id` when receives the `deleteStream` command.
    pub fn reset_message_id(&mut self) {
        self.message_id = None;
    }

    /// Gets a message ID of this stream.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_message_id().is_none())
    /// ```
    pub fn get_message_id(&mut self) -> Option<u32> {
        self.message_id
    }

    /// Sets a Playlist.
    ///
    /// Currently, this is sent from several client like OBS.
    pub fn set_playlist(&mut self, playlist: EcmaArray) {
        self.playlist = Some(playlist);
    }

    /// Gets a playlist.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_playlist().is_none())
    /// ```
    ///
    /// Currently, this is sent from several client like OBS.
    pub fn get_playlist(&mut self) -> Option<&EcmaArray> {
        self.playlist.as_ref()
    }

    /// Sets a publishing name of this stream.
    pub fn set_publishing_name(&mut self, publishing_name: AmfString) {
        self.publishing_name = Some(publishing_name);
    }

    /// Gets a publishing name of this stream.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_publishing_name().is_none())
    /// ```
    pub fn get_publishing_name(&mut self) -> Option<&AmfString> {
        self.publishing_name.as_ref()
    }

    /// Sets a publishing type of this stream.
    pub fn set_publishing_type(&mut self, publishing_type: AmfString) {
        self.publishing_type = Some(publishing_type);
    }

    /// Gets a publishing type of this stream.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_publishing_type().is_none())
    /// ```
    pub fn get_publishing_type(&mut self) -> Option<&AmfString> {
        self.publishing_type.as_ref()
    }

    /// Sets a stream name of this stream.
    pub fn set_stream_name(&mut self, stream_name: AmfString) {
        self.stream_name = Some(stream_name);
    }

    /// Gets a stream name of this stream.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_stream_name().is_none())
    /// ```
    pub fn get_stream_name(&mut self) -> Option<&AmfString> {
        self.stream_name.as_ref()
    }

    /// Sets a start time of this stream.
    pub fn set_start_time(&mut self, start_time: Option<Duration>) {
        self.start_time = start_time;
    }

    /// Gets a start time of this stream.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_start_time().is_none())
    /// ```
    pub fn get_start_time(&mut self) -> Option<Duration> {
        self.start_time
    }

    /// Sets a play mode of this stream.
    pub fn set_play_mode(&mut self, play_mode: PlayMode) {
        self.play_mode = Some(play_mode);
    }

    /// Gets a play mode of this stream.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_play_mode().is_none())
    /// ```
    pub fn get_play_mode(&mut self) -> Option<PlayMode> {
        self.play_mode
    }

    /// Sets a duration for awaiting of receiving some message.
    ///
    /// Currently, this is used only clients during publishing audio/video data.
    pub fn set_await_duration(&mut self, await_duration: Duration) {
        self.await_duration = Some(await_duration);
    }

    /// Gets a duration for awaiting of receiving some message.
    ///
    /// Currently, this is used only clients during publishing audio/video data.
    pub fn get_await_duration(&mut self) -> Option<Duration> {
        self.await_duration
    }

    /// Sets a topic file/device.
    pub fn set_topic(&mut self, topic: Flv) {
        self.topic = Some(topic);
    }

    /// Gets a topic file/device.
    ///
    /// Note this can return `None`. e.g. When this field is default as it is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_topic().is_none())
    /// ```
    pub fn get_topic(&mut self) -> Option<&Flv> {
        self.topic.as_ref()
    }

    /// Gets a topic file/device as mutable.
    ///
    /// Note this can return `None`. e.g. When it is the default as is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_topic_mut().is_none())
    /// ```
    pub fn get_topic_mut(&mut self) -> Option<&mut Flv> {
        self.topic.as_mut()
    }

    /// Stores a last received chunk.
    pub fn insert_received_chunk(&mut self, chunk_id: u16, last_chunk: LastChunk) {
        self.last_received_chunks.insert(chunk_id, last_chunk);
    }

    /// Loads a last received chunk.
    ///
    /// If no last chunk is stored associated with specified ID, this returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     handlers::RtmpContext,
    ///     messages::Channel
    /// };
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_last_received_chunk(&Channel::System.into()).is_none())
    /// ```
    pub fn get_last_received_chunk(&mut self, chunk_id: &u16) -> Option<&LastChunk> {
        self.last_received_chunks.get(chunk_id)
    }

    /// Loads a last received chunk as mutable.
    ///
    /// If no last chunk is stored associated with specified ID, this returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     handlers::RtmpContext,
    ///     messages::Channel
    /// };
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_last_received_chunk_mut(&Channel::System.into()).is_none())
    /// ```
    pub fn get_last_received_chunk_mut(&mut self, chunk_id: &u16) -> Option<&mut LastChunk> {
        self.last_received_chunks.get_mut(chunk_id)
    }

    /// Stores a last sent chunk.
    pub fn insert_sent_chunk(&mut self, chunk_id: u16, last_chunk: LastChunk) {
        self.last_sent_chunks.insert(chunk_id, last_chunk);
    }

    /// Loads a last sent chunk.
    ///
    /// If no last chunk is stored associated with specified ID, this returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     handlers::RtmpContext,
    ///     messages::Channel
    /// };
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_last_sent_chunk(&Channel::System.into()).is_none())
    /// ```
    pub fn get_last_sent_chunk(&mut self, chunk_id: &u16) -> Option<&LastChunk> {
        self.last_sent_chunks.get(chunk_id)
    }

    /// Loads a last sent chunk as mutable.
    ///
    /// If no last chunk is stored associated with specified ID, this returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     handlers::RtmpContext,
    ///     messages::Channel
    /// };
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_last_sent_chunk_mut(&Channel::System.into()).is_none())
    /// ```
    pub fn get_last_sent_chunk_mut(&mut self, chunk_id: &u16) -> Option<&mut LastChunk> {
        self.last_sent_chunks.get_mut(chunk_id)
    }
}
