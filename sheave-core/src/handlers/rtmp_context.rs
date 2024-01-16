mod last_chunk;

use std::{
    collections::HashMap,
    sync::Arc
};
use crate::{
    handshake::{
        EncryptionAlgorithm,
        Handshake
    },
    messages::{
        ChunkSize,
        amf::v0::{
            Number,
            AmfString,
            Object
        }
    }
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
#[derive(Debug, Default)]
pub struct RtmpContext {
    signed: bool,
    receiving_chunk_size: ChunkSize,
    sending_chunk_size: ChunkSize,
    last_transaction_id: Number,
    encryption_algorithm: Option<EncryptionAlgorithm>,
    client_handshake: Option<Handshake>,
    server_handshake: Option<Handshake>,
    command_object: Option<Object>,
    properties: Option<Object>,
    information: Option<Object>,
    play_path: Option<AmfString>,
    last_received_chunks: HashMap<u16, LastChunk>,
    last_sent_chunks: HashMap<u16, LastChunk>
}

impl RtmpContext {
    /// Gets a mutable reference via this wrapped by `Arc`.
    /// Sheave uses this after wrapping into `Arc`.
    /// Because of making this shareable between every handling steps.
    pub fn make_weak_mut<'a>(self: &'a Arc<Self>) -> &'a mut Self {
        unsafe { &mut *(Arc::downgrade(self).as_ptr() as *mut RtmpContext) }
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

    /// Sets a transaction ID.
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
    /// Mainly, this is used by client side contexts.
    /// Because of clients should count which transaction is it now on.
    pub fn increase_transaction_id(&mut self) {
        self.last_transaction_id += 1f64;
    }

    /// Stores the algorithm to encrypt this handshake.
    pub fn set_encryption_algorithm(&mut self, encryption_algorithm: EncryptionAlgorithm) {
        self.encryption_algorithm = Some(encryption_algorithm);
    }

    /// Gets specieifed algorithm to encrypt this handshake.
    /// Note this can return `None`. e.g. When is as the default is.
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
    /// Note this can return `None`. e.g. When is as the default is.
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
    /// Note:
    ///
    /// * This is currently used for only testing (also intagration tests contained).
    /// * This can return `None`. e.g. When is as the default is.
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
    /// Note this can return `None`. e.g. When it is the default as is.
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
    /// Note this can return `None`. e.g. When it is default as is.
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
    /// Note this can return `None`. e.g. When it is the dafault as is.
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
    /// Note this can return `None`. e.g. When it is the default as is.
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

    /// Sets a play path (e.g. filename) sent from a client.
    pub fn set_play_path(&mut self, play_path: AmfString) {
        self.play_path = Some(play_path);
    }

    /// Gets a play path (e.g. filename) sent from a client.
    /// Note this can return `None`. e.g. When it is the default as is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let mut rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_play_path().is_none())
    /// ```
    pub fn get_play_path(&mut self) -> Option<&AmfString> {
        self.play_path.as_ref()
    }

    /// Stores a last received chunk.
    pub fn insert_received_chunk(&mut self, chunk_id: u16, last_chunk: LastChunk) {
        self.last_received_chunks.insert(chunk_id, last_chunk);
    }

    /// Loads a last received chunk.
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
