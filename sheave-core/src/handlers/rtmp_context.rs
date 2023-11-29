use std::sync::Arc;
use crate::handshake::{
    EncryptionAlgorithm,
    Handshake
};

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
    encryption_algorithm: Option<EncryptionAlgorithm>,
    client_handshake: Option<Handshake>,
    server_handshake: Option<Handshake>
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
    /// use std::panic::catch_unwind;
    ///
    /// assert!(
    ///     catch_unwind(
    ///         || {
    ///             use sheave_core::handlers::RtmpContext;
    ///             RtmpContext::default().get_encryption_algorithm().unwrap()
    ///         }
    ///     ).is_err()
    /// )
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
    /// use std::panic::catch_unwind;
    ///
    /// assert!(
    ///     catch_unwind(
    ///         || {
    ///             use sheave_core::handlers::RtmpContext;
    ///
    ///             RtmpContext::default().get_client_handshake().unwrap()
    ///         }
    ///     ).is_err()
    /// )
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
    /// use std::panic::catch_unwind;
    ///
    /// assert!(
    ///     catch_unwind(
    ///         || {
    ///             use sheave_core::handlers::RtmpContext;
    ///
    ///             let mut rtmp_context = RtmpContext::default();
    ///             rtmp_context.get_client_handshake_mut().unwrap()
    ///         }
    ///     ).is_err()
    /// )
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
    /// use std::panic::catch_unwind;
    ///
    /// assert!(
    ///     catch_unwind(
    ///         || {
    ///             use sheave_core::handlers::RtmpContext;
    ///
    ///             RtmpContext::default().get_server_handshake().unwrap()
    ///         }
    ///     ).is_err()
    /// )
    /// ```
    pub fn get_server_handshake(&self) -> Option<&Handshake> {
        self.server_handshake.as_ref()
    }
}
