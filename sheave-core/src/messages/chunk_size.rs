mod negative_chunk_size;

use std::{
    cmp::Ordering,
    io::Result as IOResult
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer
};
pub use self::negative_chunk_size::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkSize(u32);

impl ChunkSize {
    const NEGATIVE_FLAG: u32 = 0x80000000;
    const DEFAULT: u32 = 128;

    pub fn new(chunk_size: u32) -> Self {
        Self(chunk_size)
    }

    pub fn get_chunk_size(&self) -> u32 {
        self.0
    }
}

impl Default for ChunkSize {
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

impl PartialOrd<u32> for ChunkSize {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<ChunkSize> for u32 {
    fn partial_cmp(&self, other: &ChunkSize) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Decoder<ChunkSize> for ByteBuffer {
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
    fn encode(&mut self, chunk_size: &ChunkSize) {
        self.put_u32_be(chunk_size.0);
    }
}
