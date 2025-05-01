//! # Types for the handshake step in RTMP.
//!
//! In RTMP, first, both sides are required doing handshake.
//! It is done respectively following steps:
//!
//! 1. Specifies RTMP version.
//! 2. Exchanges handshake data each other.
//! 3. Returns partner's handshake data.
//!
//! ## RTMP version
//!
//! 1 byte to specify a kind of encryption.
//! Default is 3.
//! This means doing handshake as the Raw RTMP, that is, not to encrypt.
//! Server should respond 3 if encryption specified by client has not implemented.
//! In this case, client can either degrade version to 3 or disconnect with server.
//!
//! ## Handshake
//!
//! 1536 bytes of actual handshake data.
//! Note this can be imprinted HMAC-SHA256 diegst/signature according to version of Flash Player/Flash Media Server.
//! Concretely, it is imprinted when respective version is following:
//!
//! * Flash Player: `>= 9`
//! * Flash Media Server: `>= 3`
//!
//! ### Examples
//!
//! Both sides are required taking following steps each version.
//!
//! * Below Flash Player 9/Flash Media Server 3
//!
//! ```rust
//! use std::time::Duration;
//! use sheave_core::handshake::{
//!     Handshake,
//!     Version
//! };
//!
//! let handshake = Handshake::new(Duration::default(), Version::UNSIGNED);
//! ```
//!
//! * And above Flash Player 9/Flash Media Server 3
//!
//! ```rust
//! use std::time::Duration;
//! use sheave_core::handshake::{
//!     Handshake,
//!     Version,
//!     EncryptionAlgorithm
//! };
//!
//! // In a case of exchanging client-side request with server-side response.
//! let mut client_handshake = Handshake::new(Duration::default(), Version::LATEST_CLIENT);
//! client_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY);
//! let mut key: Vec<u8> = Vec::new();
//! key.extend_from_slice(Handshake::SERVER_KEY);
//! key.extend_from_slice(Handshake::COMMON_KEY);
//! client_handshake.imprint_signature(EncryptionAlgorithm::NotEncrypted, key.as_slice());
//! assert!(client_handshake.did_signature_match(EncryptionAlgorithm::NotEncrypted, key.as_slice()));
//!
//! // In a case of exchanging server-side request with client-side response.
//! let mut server_handshake = Handshake::new(Duration::default(), Version::LATEST_SERVER);
//! server_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY);
//! let mut key: Vec<u8> = Vec::new();
//! key.extend_from_slice(Handshake::CLIENT_KEY);
//! key.extend_from_slice(Handshake::COMMON_KEY);
//! server_handshake.imprint_signature(EncryptionAlgorithm::NotEncrypted, key.as_slice());
//! assert!(server_handshake.did_signature_match(EncryptionAlgorithm::NotEncrypted, key.as_slice()));
//! ```
//!
//! ### Encryption
//!
//! Currently, to implement handshake encryption isn't planned following causes:
//!
//! 1. Connected socket is in full view from outside. This is insecure though chunk is encrypted.
//! 2. If chunk encryption is implemented on RTMPTS, To decrypt chunk/socket takes both sides time in no small way. This is inefficient for real-time communications.
//! 3. Therefore I'm thinking we should leave encryption to only HTTPS.

mod version;
mod encryption_algorithm;

use std::time::Duration;
use rand::fill;
use digest::{
    CtOutput,
    OutputSizeUser
};
use sha2::Sha256;
use hmac::{
    Hmac,
    Mac
};
pub use self::{
    version::Version,
    encryption_algorithm::EncryptionAlgorithm
};

type HmacSha256 = Hmac<Sha256>;

/// The 1536 bytes handshake data.
/// This respectively consists of following parts:
///
/// |Range|Representation|
/// | :- | :- |
/// |First 4 bytes|Timestamp (in milliseconds)|
/// |Second 4 bytes|Flash Player version/Flash Media Server version|
/// |Remained bytes|Randoms for handshake (may be contained digest/signature)|
#[derive(Debug)]
pub struct Handshake([u8; 1536]);

impl Handshake {
    /// The key which is used to imprint any client-side digest.
    pub const CLIENT_KEY: &'static [u8] = b"Genuine Adobe Flash Player 001";
    /// The key which is used to imprint any server-side digest.
    pub const SERVER_KEY: &'static [u8] = b"Genuine Adobe Flash Media Server 001";
    /// The key which is used to imprint any signature.
    /// Both sides are required to contain this into a key of signature.
    pub const COMMON_KEY: &'static [u8] = &[
        0xF0, 0xEE, 0xC2, 0x4A, 0x80, 0x68, 0xBE, 0xE8, 0x2E, 0x00, 0xD0, 0xD1, 0x02, 0x9E, 0x7E, 0x57, 0x6E, 0xEC, 0x5D, 0x2D, 0x29, 0x80, 0x6F, 0xAB, 0x93, 0xB8, 0xE6, 0x36, 0xCF, 0xEB, 0x31, 0xAE
    ];

    /// Constructs handshake data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::handshake::{
    ///     Handshake,
    ///     Version,
    ///     EncryptionAlgorithm
    /// };
    ///
    /// // If you are a client.
    /// let mut client_handshake = Handshake::new(Duration::default(), Version::LATEST_CLIENT);
    /// // If you are a server.
    /// let mut server_handshake = Handshake::new(Duration::default(), Version::LATEST_SERVER);
    /// ```
    pub fn new(timestamp: Duration, version: Version) -> Self {
        let mut handshake_bytes: [u8; 1536] = [0; 1536];
        let timestamp_bytes = (timestamp.as_millis() as u32).to_be_bytes();
        let version_bytes: [u8; 4] = version.into();
        handshake_bytes[..4].copy_from_slice(timestamp_bytes.as_slice());
        handshake_bytes[4..8].copy_from_slice(version_bytes.as_slice());
        fill(&mut handshake_bytes[8..]);
        Self(handshake_bytes)
    }

    /// Gets all handshake data.
    pub fn get_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Gets first 4 bytes as timestamp.
    pub fn get_timestamp(&self) -> Duration {
        let mut timestamp_bytes: [u8; 4] = [0; 4];
        timestamp_bytes.copy_from_slice(&self.0[..4]);
        Duration::from_millis(u32::from_be_bytes(timestamp_bytes) as u64)
    }

    /// Gets second 4 bytes as Flash Player version/Flash Media Server version.
    pub fn get_version(&self) -> Version {
        let mut version_bytes: [u8; 4] = [0; 4];
        version_bytes.copy_from_slice(&self.0[4..8]);
        version_bytes.into()
    }

    fn get_digest_position(&self, encryption_algorithm: EncryptionAlgorithm) -> usize {
        let offset: usize;
        let adder: usize;
        match encryption_algorithm {
            EncryptionAlgorithm::NotEncrypted => {
                offset = 8;
                adder = 12;
            },
            _ => {
                offset = 772;
                adder = 776;
            }
        }
        self.0[offset..(offset + 4)].iter().map(|byte| usize::from(*byte)).sum::<usize>() % 728 + adder
    }

    fn get_digest_message(&self, encryption_algorithm: EncryptionAlgorithm) -> Vec<u8> {
        let digest_position = self.get_digest_position(encryption_algorithm);
        let mut message: Vec<u8> = Vec::new();
        message.extend_from_slice(&self.0[..digest_position]);
        message.extend_from_slice(&self.0[(digest_position + HmacSha256::output_size())..]);
        message
    }

    fn compute_digest(&self, encryption_algorithm: EncryptionAlgorithm, key: &[u8]) -> CtOutput<HmacSha256> {
        let message = self.get_digest_message(encryption_algorithm);
        let mut hmac = HmacSha256::new_from_slice(key).unwrap();
        hmac.update(message.as_slice());
        hmac.finalize()
    }

    /// Gets a digest contained in this handshake bytes.
    /// Note its place is different by whether encrypts handshake bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Instant;
    /// use sheave_core::handshake::{
    ///     EncryptionAlgorithm,
    ///     Handshake,
    ///     Version
    /// };
    ///
    /// let handshake = Handshake::new(Instant::now().elapsed(), Version::LATEST_CLIENT);
    ///
    /// assert_ne!(handshake.get_digest(EncryptionAlgorithm::NotEncrypted), handshake.get_digest(EncryptionAlgorithm::DiffieHellman))
    /// ```
    pub fn get_digest(&self, encryption_algorithm: EncryptionAlgorithm) -> &[u8] {
        let digest_position = self.get_digest_position(encryption_algorithm);
        &self.0[digest_position..(digest_position + HmacSha256::output_size())]
    }

    /// Imprints an HMAC-SHA256 digest into handshake data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::handshake::{
    ///     Handshake,
    ///     Version,
    ///     EncryptionAlgorithm
    /// };
    ///
    /// // In a case of sending client-side request.
    /// let mut client_handshake = Handshake::new(Duration::default(), Version::LATEST_CLIENT);
    /// client_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY);
    ///
    /// // In a case of sending server-side request.
    /// let mut server_handshake = Handshake::new(Duration::default(), Version::LATEST_SERVER);
    /// server_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY);
    /// ```
    pub fn imprint_digest(&mut self, encryption_algorithm: EncryptionAlgorithm, key: &[u8]) {
        let digest_position = self.get_digest_position(encryption_algorithm);
        let digest = self.compute_digest(encryption_algorithm, key);
        self.0[digest_position..(digest_position + HmacSha256::output_size())].copy_from_slice(digest.into_bytes().as_slice());
    }

    /// Checks whether imprinted digest matches with one computed by given key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::handshake::{
    ///     Handshake,
    ///     Version,
    ///     EncryptionAlgorithm
    /// };
    ///
    /// // In a case of checking server-side request.
    /// let mut server_handshake = Handshake::new(Duration::default(), Version::LATEST_SERVER);
    /// server_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY);
    /// assert!(server_handshake.did_digest_match(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY));
    ///
    /// // In a case of checking client-side request.
    /// let mut client_handshake = Handshake::new(Duration::default(), Version::LATEST_CLIENT);
    /// server_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY);
    /// assert!(server_handshake.did_digest_match(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY));
    /// ```
    pub fn did_digest_match(&self, encryption_algorithm: EncryptionAlgorithm, key: &[u8]) -> bool {
        let expected = self.compute_digest(encryption_algorithm, key);
        self.get_digest(encryption_algorithm) == expected.into_bytes().as_slice()
    }

    fn get_signature_position(&self) -> usize {
        self.0.len() - HmacSha256::output_size()
    }

    fn get_signature_message(&self) -> &[u8] {
        let signature_position = self.get_signature_position();
        &self.0[..signature_position]
    }

    fn compute_signature(&self, encryption_algorithm: EncryptionAlgorithm, key: &[u8]) -> CtOutput<HmacSha256> {
        let digest = self.get_digest(encryption_algorithm);
        let mut hmac = HmacSha256::new_from_slice(key).unwrap();
        hmac.update(digest);
        let key_from_digest = hmac.finalize();

        let message = self.get_signature_message();
        let mut hmac = HmacSha256::new_from_slice(key_from_digest.into_bytes().as_slice()).unwrap();
        hmac.update(message);
        hmac.finalize()
    }

    /// Gets a signature contained into this handshake bytes.
    pub fn get_signature(&self) -> &[u8] {
        let signature_position = self.get_signature_position();
        &self.0[signature_position..]

    }

    /// Imprints an HMAC-SHA256 signature into handshake data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::handshake::{
    ///     Handshake,
    ///     Version,
    ///     EncryptionAlgorithm
    /// };
    ///
    /// // In a case of exchanging client-side request with server-side response.
    /// let mut client_handshake = Handshake::new(Duration::default(), Version::LATEST_CLIENT);
    /// client_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY);
    /// let mut key: Vec<u8> = Vec::new();
    /// key.extend_from_slice(Handshake::SERVER_KEY);
    /// key.extend_from_slice(Handshake::COMMON_KEY);
    /// client_handshake.imprint_signature(EncryptionAlgorithm::NotEncrypted, key.as_slice());
    ///
    /// // In a case of exchanging server-side request with client-side response.
    /// let mut server_handshake = Handshake::new(Duration::default(), Version::LATEST_SERVER);
    /// server_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY);
    /// let mut key: Vec<u8> = Vec::new();
    /// key.extend_from_slice(Handshake::CLIENT_KEY);
    /// key.extend_from_slice(Handshake::COMMON_KEY);
    /// server_handshake.imprint_signature(EncryptionAlgorithm::NotEncrypted, key.as_slice());
    /// ```
    pub fn imprint_signature(&mut self, encryption_algorithm: EncryptionAlgorithm, key: &[u8]) {
        let signature_position = self.get_signature_position();
        let signature = self.compute_signature(encryption_algorithm, key);
        self.0[signature_position..].copy_from_slice(signature.into_bytes().as_slice());
    }

    /// Checks whether imprinted signature matches one computed by given key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::handshake::{
    ///     Handshake,
    ///     Version,
    ///     EncryptionAlgorithm
    /// };
    ///
    /// // In a case of checking client-side response.
    /// let mut client_handshake = Handshake::new(Duration::default(), Version::LATEST_CLIENT);
    /// client_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY);
    /// let mut key: Vec<u8> = Vec::new();
    /// key.extend_from_slice(Handshake::SERVER_KEY);
    /// key.extend_from_slice(Handshake::COMMON_KEY);
    /// client_handshake.imprint_signature(EncryptionAlgorithm::NotEncrypted, key.as_slice());
    /// assert!(client_handshake.did_signature_match(EncryptionAlgorithm::NotEncrypted, key.as_slice()));
    ///
    /// // In a case of checking server-side response.
    /// let mut server_handshake = Handshake::new(Duration::default(), Version::LATEST_SERVER);
    /// server_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY);
    /// let mut key: Vec<u8> = Vec::new();
    /// key.extend_from_slice(Handshake::CLIENT_KEY);
    /// key.extend_from_slice(Handshake::COMMON_KEY);
    /// server_handshake.imprint_signature(EncryptionAlgorithm::NotEncrypted, key.as_slice());
    /// assert!(server_handshake.did_signature_match(EncryptionAlgorithm::NotEncrypted, key.as_slice()));
    /// ```
    pub fn did_signature_match(&self, encryption_algorithm: EncryptionAlgorithm, key: &[u8]) -> bool {
        let expected = self.compute_signature(encryption_algorithm, key);
        self.get_signature() == expected.into_bytes().as_slice()
    }
}

impl From<[u8; 1536]> for Handshake {
    fn from(handshake_bytes: [u8; 1536]) -> Self {
        Self(handshake_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn did_client_digest_match() {
        let mut client_handshake = Handshake::new(Duration::default(), Version::LATEST_CLIENT);
        client_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY);
        assert!(client_handshake.did_digest_match(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY))
    }

    #[test]
    fn did_server_digest_match() {
        let mut server_handshake = Handshake::new(Duration::default(), Version::LATEST_SERVER);
        server_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY);
        assert!(server_handshake.did_digest_match(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY))
    }

    #[test]
    fn did_client_signature_match() {
        let mut client_handshake = Handshake::new(Duration::default(), Version::LATEST_CLIENT);
        client_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY);
        let mut key: Vec<u8> = Vec::new();
        key.extend_from_slice(Handshake::SERVER_KEY);
        key.extend_from_slice(Handshake::COMMON_KEY);
        client_handshake.imprint_signature(EncryptionAlgorithm::NotEncrypted, key.as_slice());
        assert!(client_handshake.did_signature_match(EncryptionAlgorithm::NotEncrypted, key.as_slice()))
    }

    #[test]
    fn did_server_signature_match() {
        let mut server_handshake = Handshake::new(Duration::default(), Version::LATEST_SERVER);
        server_handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::SERVER_KEY);
        let mut key: Vec<u8> = Vec::new();
        key.extend_from_slice(Handshake::CLIENT_KEY);
        key.extend_from_slice(Handshake::COMMON_KEY);
        server_handshake.imprint_signature(EncryptionAlgorithm::NotEncrypted, key.as_slice());
        assert!(server_handshake.did_signature_match(EncryptionAlgorithm::NotEncrypted, key.as_slice()))
    }
}
