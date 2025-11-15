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

/// The message to tell the bandwidth limit as a receiver side.
///
/// Following format is required.
///
/// |Field|Length (in bytes)|Description|
/// | :- | -: | :- |
/// |Bandwidth|4|The bandwidth number of a receiver side.|
///
/// When some amount of receiving data got exceeded this bandwidth, a receiver must send about it to a sender, by the [`Acknowledgement`] message.
///
/// Almost RTMP tools are specifying it 2500000 in bits as their defaults.
///
/// [`Acknowledgement`]: crate::messages::Acknowledgement
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowAcknowledgementSize(u32);

impl WindowAcknowledgementSize {
    const DEFAULT: u32 = 2500000;

    /// Constructs an WindowAcknowledgementSize message.
    pub fn new(window_acknowledgement_size: u32) -> Self {
        Self(window_acknowledgement_size)
    }

    /// Gets an inner value.
    pub fn get_inner(&self) -> u32 {
        self.0
    }
}

impl Default for WindowAcknowledgementSize {
    /// Constructs an WindowAcknowledgementSize message with the default bandwidth (2500000 in bits).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::WindowAcknowledgementSize;
    ///
    /// assert_eq!(2500000u32, WindowAcknowledgementSize::default())
    /// ```
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

impl PartialEq<u32> for WindowAcknowledgementSize {
    fn eq(&self, other: &u32) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<WindowAcknowledgementSize> for u32 {
    fn eq(&self, other: &WindowAcknowledgementSize) -> bool {
        self.eq(&other.0)
    }
}

impl PartialOrd<u32> for WindowAcknowledgementSize {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<WindowAcknowledgementSize> for u32 {
    fn partial_cmp(&self, other: &WindowAcknowledgementSize) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Div<u32> for WindowAcknowledgementSize {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ChunkData for WindowAcknowledgementSize {
    const CHANNEL: Channel = Channel::Network;
    const MESSAGE_TYPE: MessageType = MessageType::WindowAcknowledgementSize;
}

impl Decoder<WindowAcknowledgementSize> for ByteBuffer {
    /// Decodes bytes into an WindowAcknowledgementSize message.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When this buffer didn't remain at least 4 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     messages::WindowAcknowledgementSize
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u32_be(u32::default());
    /// assert!(Decoder::<WindowAcknowledgementSize>::decode(&mut buffer).is_ok())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    fn decode(&mut self) -> IOResult<WindowAcknowledgementSize> {
        self.get_u32_be().map(WindowAcknowledgementSize::new)
    }
}

impl Encoder<WindowAcknowledgementSize> for ByteBuffer {
    /// Encodes an WindowAcknowledgementSize message into bytes.
    fn encode(&mut self, window_acknowledgement_size: &WindowAcknowledgementSize) {
        self.put_u32_be(window_acknowledgement_size.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_window_acknowledgement_size() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u32_be(2500000);
        let result: IOResult<WindowAcknowledgementSize> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = WindowAcknowledgementSize::default();
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_window_acknowledgement_size() {
        let mut buffer = ByteBuffer::default();
        let expected_bytes = 2500000u32;
        buffer.encode(&WindowAcknowledgementSize::default());
        let actual_bytes = buffer.get_u32_be().unwrap();
        assert_eq!(expected_bytes, actual_bytes)
    }
}
