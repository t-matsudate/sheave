mod negative_chunk_size;

use std::io::Result as IOResult;
use super::{
    Channel,
    ChunkData,
    headers::MessageType
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer
};
pub use self::negative_chunk_size::*;

/// Tells a size to chunk its stream to the partner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkSize(u32);

impl ChunkSize {
    const NEGATIVE_FLAG: u32 = 0x80000000;
    const DEFAULT: u32 = 128;

    /// Constructs a chunk size.
    pub fn new(chunk_size: u32) -> Self {
        Self(chunk_size)
    }

    /// Gets an internal value.
    pub fn get_chunk_size(&self) -> u32 {
        self.0
    }
}

impl Default for ChunkSize {
    /// Constructs a ChunkSize with its default value.
    /// In RTMP, the default value of chunking size is defined to be 128.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::ChunkSize;
    ///
    /// let chunk_size = ChunkSize::default();
    /// assert_eq!(128, chunk_size)
    /// ```
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

impl PartialEq<u32> for ChunkSize {
    fn eq(&self, other: &u32) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<ChunkSize> for u32 {
    fn eq(&self, other: &ChunkSize) -> bool {
        self.eq(&other.0)
    }
}

impl ChunkData for ChunkSize {
    const CHANNEL: Channel = Channel::Network;
    const MESSAGE_TYPE: MessageType = MessageType::ChunkSize;
}

impl Decoder<ChunkSize> for ByteBuffer {
    /// Decodes bytes into a ChunkSize.
    ///
    /// # Errors
    ///
    /// * [`Insufficientbufferlength`]
    ///
    /// When chunk data didn't remain at least 4 bytes.
    ///
    /// * [`NegativeChunkSize`]
    ///
    /// When its received chunk size's most significant bit is 1.
    /// Chunking size is required that its bit is 0 in the specification.
    /// This is probably considered of programs which has no unsigned type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     messages::ChunkSize
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u32_be(128);
    /// assert!(Decoder::<ChunkSize>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u32_be(0x80000000);
    /// assert!(Decoder::<ChunkSize>::decode(&mut buffer).is_err());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<ChunkSize>::decode(&mut buffer).is_err());
    /// ```
    ///
    /// [`Insufficientbufferlength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`NegativeChunkSize`]: NegativeChunkSize
    fn decode(&mut self) -> IOResult<ChunkSize> {
        let chunk_size = self.get_u32_be()?;

        if chunk_size & ChunkSize::NEGATIVE_FLAG != 0 {
            Err(negative_chunk_size(chunk_size))
        } else {
            Ok(ChunkSize(chunk_size))
        }
    }
}

impl Encoder<ChunkSize> for ByteBuffer {
    /// Encodes a ChunkSize into bytes.
    fn encode(&mut self, chunk_size: &ChunkSize) {
        self.put_u32_be(chunk_size.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_chunk_size() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u32_be(128);
        assert!(Decoder::<ChunkSize>::decode(&mut buffer).is_ok());

        let mut buffer = ByteBuffer::default();
        buffer.put_u32_be(0x80000000);
        assert!(Decoder::<ChunkSize>::decode(&mut buffer).is_err())
    }

    #[test]
    fn encode_chunk_size() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&ChunkSize::default());
        let chunk_size = buffer.get_u32_be().unwrap();
        assert_eq!(128, chunk_size)
    }
}
