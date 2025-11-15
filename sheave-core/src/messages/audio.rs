use std::{
    borrow::Cow,
    collections::{
        BinaryHeap,
        VecDeque
    },
    ffi::CString,
    io::Result as IOResult,
    ops::{
        Deref,
        DerefMut,
        Index,
        IndexMut
    },
    rc::Rc,
    slice::SliceIndex,
    sync::Arc
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer
};
use super::{
    Channel,
    ChunkData,
    headers::MessageType
};

/// The message to handle something audio data.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Audio(Vec<u8>);

impl Audio {
    /// Constructs a new audio data.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Gets an internal byte array.
    pub fn get_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> From<[u8; N]> for Audio {
    fn from(bytes: [u8; N]) -> Self {
        Self(bytes.into())
    }
}

impl<const N: usize> From<&[u8; N]> for Audio {
    fn from(bytes: &[u8; N]) -> Self {
        Self(bytes.into())
    }
}

impl<const N: usize> From<&mut [u8; N]> for Audio {
    fn from(bytes: &mut [u8; N]) -> Self {
        Self(bytes.into())
    }
}

impl From<&[u8]> for Audio {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.into())
    }
}

impl From<&mut [u8]> for Audio {
    fn from(bytes: &mut [u8]) -> Self {
        Self(bytes.into())
    }
}

impl From<&str> for Audio {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<String> for Audio {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<CString> for Audio {
    fn from(s: CString) -> Self {
        Self(s.into())
    }
}

impl From<BinaryHeap<u8>> for Audio {
    fn from(bytes: BinaryHeap<u8>) -> Self {
        Self(bytes.into())
    }
}

impl From<Audio> for BinaryHeap<u8> {
    fn from(audio: Audio) -> Self {
        audio.0.into()
    }
}

impl From<Box<[u8]>> for Audio {
    fn from(bytes: Box<[u8]>) -> Self {
        Self(bytes.into())
    }
}

impl From<Audio> for Box<[u8]> {
    fn from(audio: Audio) -> Self {
        audio.0.into()
    }
}

impl<'a> From<Cow<'a, [u8]>> for Audio {
    fn from(bytes: Cow<'a, [u8]>) -> Self {
        Self(bytes.into())
    }
}

impl<'a> From<Audio> for Cow<'a, [u8]> {
    fn from(audio: Audio) -> Self {
        audio.0.into()
    }
}

impl<'a> From<&'a Audio> for Cow<'a, [u8]> {
    fn from(audio: &'a Audio) -> Self {
        Cow::from(&audio.0)
    }
}

impl From<VecDeque<u8>> for Audio {
    fn from(bytes: VecDeque<u8>) -> Self {
        Self(bytes.into())
    }
}

impl From<Audio> for VecDeque<u8> {
    fn from(audio: Audio) -> Self {
        audio.0.into()
    }
}

impl From<Audio> for Vec<u8> {
    fn from(audio: Audio) -> Self {
        audio.0
    }
}

impl From<Audio> for Rc<[u8]> {
    fn from(audio: Audio) -> Self {
        audio.0.into()
    }
}

impl From<Audio> for Arc<[u8]> {
    fn from(audio: Audio) -> Self {
        audio.0.into()
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for Audio {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for Audio {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl Deref for Audio {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Audio {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> PartialEq<[u8; N]> for Audio {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.0.eq(other)
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for Audio {
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<[u8]> for Audio {
    fn eq(&self, other: &[u8]) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Audio> for [u8] {
    fn eq(&self, other: &Audio) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<&[u8]> for Audio {
    fn eq(&self, other: &&[u8]) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Audio> for &[u8] {
    fn eq(&self, other: &Audio) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<&mut [u8]> for Audio {
    fn eq(&self, other: &&mut [u8]) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Audio> for &mut [u8] {
    fn eq(&self, other: &Audio) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<Vec<u8>> for Audio {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Audio> for Vec<u8> {
    fn eq(&self, other: &Audio) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<Audio> for Cow<'_, [u8]> {
    fn eq(&self, other: &Audio) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<Audio> for VecDeque<u8> {
    fn eq(&self, other: &Audio) -> bool {
        self.eq(&other.0)
    }
}

impl ChunkData for Audio {
    const CHANNEL: Channel = Channel::Audio;
    const MESSAGE_TYPE: MessageType = MessageType::Audio;
}

impl Decoder<Audio> for ByteBuffer {
    fn decode(&mut self) -> IOResult<Audio> {
        let remained = self.remained();
        self.get_bytes(remained).map(|bytes| Audio::new(bytes.to_vec()))
    }
}

impl Encoder<Audio> for ByteBuffer {
    fn encode(&mut self, audio: &Audio) {
        self.put_bytes(audio.get_bytes());
    }
}
