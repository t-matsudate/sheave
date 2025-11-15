mod limit_type;

use std::{
    cmp::Ordering,
    io::Result as IOResult,
    ops::Div
};
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::{
        Channel,
        ChunkData,
        headers::MessageType
    }
};
pub use self::limit_type::*;

/// The message to tell the client-side bandwidth.
///
/// This has 2 ways of comparision by which field you specifies.
///
/// # Examples
///
/// ```rust
/// use sheave_core::messages::{
///     LimitType,
///     PeerBandwidth,
/// };
///
/// let peer_bandwidth = PeerBandwidth::default();
///
/// // When you compare this message with a bandwidth number.
/// assert!(2500000u32 == peer_bandwidth);
/// assert!(0 <= peer_bandwidth);
/// // When you compare this message with a limit type.
/// assert!(LimitType::default() == peer_bandwidth)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PeerBandwidth(u32, LimitType);

impl PeerBandwidth {
    const DEFAULT: u32 = 2500000;

    /// Constructs a PeerBandwidth message.
    pub fn new(peer_bandwidth: u32, limit_type: LimitType) -> Self {
        Self(peer_bandwidth, limit_type)
    }

    pub fn get_inner_bandwidth(&self) -> u32 {
        self.0
    }

    pub fn get_inner_limit_type(&self) -> LimitType {
        self.1
    }
}

impl Default for PeerBandwidth {
    /// Constructs a PeerBandwidth message with the default bandwidth and the default limit type (2500000 in bits, 2 (Dynamic)).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::{
    ///     LimitType,
    ///     PeerBandwidth
    /// };
    ///
    /// let peer_bandwidth = PeerBandwidth::default();
    /// assert_eq!(2500000u32, peer_bandwidth);
    /// assert_eq!(LimitType::default(), peer_bandwidth)
    /// ```
    fn default() -> Self {
        Self(Self::DEFAULT, LimitType::default())
    }
}

impl PartialEq<u32> for PeerBandwidth {
    fn eq(&self, other: &u32) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<PeerBandwidth> for u32 {
    fn eq(&self, other: &PeerBandwidth) -> bool {
        self.eq(&other.0)
    }
}

impl PartialOrd<u32> for PeerBandwidth {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<PeerBandwidth> for u32 {
    fn partial_cmp(&self, other: &PeerBandwidth) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl PartialEq<LimitType> for PeerBandwidth {
    fn eq(&self, other: &LimitType) -> bool {
        self.1.eq(other)
    }
}

impl PartialEq<PeerBandwidth> for LimitType {
    fn eq(&self, other: &PeerBandwidth) -> bool {
        self.eq(&other.1)
    }
}

impl Div<u32> for PeerBandwidth {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Self(self.0 / rhs, self.1)
    }
}

impl ChunkData for PeerBandwidth {
    const CHANNEL: Channel = Channel::Network;
    const MESSAGE_TYPE: MessageType = MessageType::PeerBandwidth;
}

impl Decoder<PeerBandwidth> for ByteBuffer {
    /// Decodes bytes into a PeerBandwidth message.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When this buffer didn't remain at least 5 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     messages::PeerBandwidth
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u32_be(u32::default());
    /// buffer.put_u8(u8::default());
    /// assert!(Decoder::<PeerBandwidth>::decode(&mut buffer).is_ok())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    fn decode(&mut self) -> IOResult<PeerBandwidth> {
        let bandwidth = self.get_u32_be()?;
        let limit_type = self.get_u8()?;
        Ok(PeerBandwidth(bandwidth, limit_type.into()))
    }
}

impl Encoder<PeerBandwidth> for ByteBuffer {
    /// Encodes a PeerBandwidth message into bytes.
    fn encode(&mut self, peer_bandwidth: &PeerBandwidth) {
        self.put_u32_be(peer_bandwidth.0);
        self.put_u8(peer_bandwidth.1.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_peer_bandwidth() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u32_be(2500000);
        buffer.put_u8(2);
        let result: IOResult<PeerBandwidth> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = PeerBandwidth::default();
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_peer_bandwidth() {
        let mut buffer = ByteBuffer::default();
        let expected_bandwidth = 2500000u32;
        let expected_limit_type = 2u8;
        buffer.encode(&PeerBandwidth::default());
        let actual_bandwidth = buffer.get_u32_be().unwrap();
        assert_eq!(expected_bandwidth, actual_bandwidth);
        let actual_limit_type = buffer.get_u8().unwrap();
        assert_eq!(expected_limit_type, actual_limit_type)
    }
}
