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
    messages::ChunkSize
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
    encryption_algorithm: Option<EncryptionAlgorithm>,
    client_handshake: Option<Handshake>,
    server_handshake: Option<Handshake>,
    last_received_chunks: HashMap<u16, LastChunk>,
    last_sent_chunks: HashMap<u16, LastChunk>
}

impl RtmpContext {
    pub fn make_weak_mut<'a>(self: &'a Arc<Self>) -> &'a mut Self {
        unsafe { &mut *(Arc::downgrade(self).as_ptr() as *mut RtmpContext) }
    }

    /// Stores a flag to mean this handshake is signed.
    pub fn set_signed(&mut self, signed: bool) {
        self.signed = signed;
    }

    /// Indicates whether the handshake is signed.
    pub fn is_signed(&self) -> bool {
        self.signed
    }

    pub fn set_receiving_chunk_size(&mut self, chunk_size: ChunkSize) {
        self.receiving_chunk_size = chunk_size;
    }

    pub fn get_receiving_chunk_size(&self) -> &ChunkSize {
        &self.receiving_chunk_size
    }

    pub fn set_sending_chunk_size(&mut self, chunk_size: ChunkSize) {
        self.sending_chunk_size = chunk_size;
    }

    pub fn get_sending_chunk_size(&self) -> &ChunkSize {
        &self.sending_chunk_size
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
    /// let rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_encryption_algorithm().is_none())
    /// ```
    pub fn get_encryption_algorithm(&self) -> Option<EncryptionAlgorithm> {
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
    /// let rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_client_handshake().is_none())
    /// ```
    pub fn get_client_handshake(&self) -> Option<&Handshake> {
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
    /// Note this can return `None`. e.g. When is the default is.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handlers::RtmpContext;
    ///
    /// let rtmp_context = RtmpContext::default();
    /// assert!(rtmp_context.get_server_handshake().is_none())
    /// ```
    pub fn get_server_handshake(&self) -> Option<&Handshake> {
        self.server_handshake.as_ref()
    }

    pub fn insert_received_chunk(&mut self, chunk_id: u16, last_chunk: LastChunk) {
        self.last_received_chunks.insert(chunk_id, last_chunk);
    }

    pub fn get_last_received_chunk(&self, chunk_id: &u16) -> Option<&LastChunk> {
        self.last_received_chunks.get(chunk_id)
    }

    pub fn get_last_received_chunk_mut(&mut self, chunk_id: &u16) -> Option<&mut LastChunk> {
        self.last_received_chunks.get_mut(chunk_id)
    }

    pub fn insert_sent_chunk(&mut self, chunk_id: u16, last_chunk: LastChunk) {
        self.last_sent_chunks.insert(chunk_id, last_chunk);
    }

    pub fn get_last_sent_chunk(&self, chunk_id: &u16) -> Option<&LastChunk> {
        self.last_sent_chunks.get(chunk_id)
    }

    pub fn get_last_sent_chunk_mut(&mut self, chunk_id: &u16) -> Option<&mut LastChunk> {
        self.last_sent_chunks.get_mut(chunk_id)
    }
}
