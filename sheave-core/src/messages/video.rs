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

/// The message to handle something video data.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Video(Vec<u8>);

impl Video {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> From<[u8; N]> for Video {
    fn from(bytes: [u8; N]) -> Self {
        Self(bytes.into())
    }
}

impl<const N: usize> From<&[u8; N]> for Video {
    fn from(bytes: &[u8; N]) -> Self {
        Self(bytes.into())
    }
}

impl<const N: usize> From<&mut [u8; N]> for Video {
    fn from(bytes: &mut [u8; N]) -> Self {
        Self(bytes.into())
    }
}

impl From<&[u8]> for Video {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.into())
    }
}

impl From<&mut [u8]> for Video {
    fn from(bytes: &mut [u8]) -> Self {
        Self(bytes.into())
    }
}

impl From<&str> for Video {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<String> for Video {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<CString> for Video {
    fn from(s: CString) -> Self {
        Self(s.into())
    }
}

impl<'a> From<Cow<'a, [u8]>> for Video {
    fn from(bytes: Cow<'a, [u8]>) -> Self {
        Self(bytes.into())
    }
}

impl From<Box<[u8]>> for Video {
    fn from(bytes: Box<[u8]>) -> Self {
        Self(bytes.into())
    }
}

impl From<VecDeque<u8>> for Video {
    fn from(bytes: VecDeque<u8>) -> Self {
        Self(bytes.into())
    }
}

impl From<BinaryHeap<u8>> for Video {
    fn from(bytes: BinaryHeap<u8>) -> Self {
        Self(bytes.into())
    }
}

impl From<Video> for Vec<u8> {
    fn from(video: Video) -> Self {
        video.0
    }
}

impl<'a> From<Video> for Cow<'a, [u8]> {
    fn from(video: Video) -> Self {
        video.0.into()
    }
}

impl<'a> From<&'a Video> for Cow<'a, [u8]> {
    fn from(video: &'a Video) -> Self {
        Cow::from(&video.0)
    }
}

impl From<Video> for Box<[u8]> {
    fn from(video: Video) -> Self {
        video.0.into()
    }
}

impl From<Video> for VecDeque<u8> {
    fn from(video: Video) -> Self {
        video.0.into()
    }
}

impl From<Video> for BinaryHeap<u8> {
    fn from(video: Video) -> Self {
        video.0.into()
    }
}

impl From<Video> for Rc<[u8]> {
    fn from(video: Video) -> Self {
        video.0.into()
    }
}

impl From<Video> for Arc<[u8]> {
    fn from(video: Video) -> Self {
        video.0.into()
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for Video {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for Video {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl Deref for Video {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Video {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> PartialEq<[u8; N]> for Video {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.0.eq(other)
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for Video {
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<[u8]> for Video {
    fn eq(&self, other: &[u8]) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<&[u8]> for Video {
    fn eq(&self, other: &&[u8]) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<&mut [u8]> for Video {
    fn eq(&self, other: &&mut [u8]) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Vec<u8>> for Video {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Video> for [u8] {
    fn eq(&self, other: &Video) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<Video> for &[u8] {
    fn eq(&self, other: &Video) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<Video> for &mut [u8] {
    fn eq(&self, other: &Video) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<Video> for Cow<'_, [u8]> {
    fn eq(&self, other: &Video) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<Video> for Vec<u8> {
    fn eq(&self, other: &Video) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<Video> for VecDeque<u8> {
    fn eq(&self, other: &Video) -> bool {
        self.eq(&other.0)
    }
}

impl ChunkData for Video {
    const CHANNEL: Channel = Channel::Video;
    const MESSAGE_TYPE: MessageType = MessageType::Video;
}

impl Decoder<Video> for ByteBuffer {
    fn decode(&mut self) -> IOResult<Video> {
        let remained = self.remained();
        self.get_bytes(remained).map(|bytes| Video::new(bytes.to_vec()))
    }
}

impl Encoder<Video> for ByteBuffer {
    fn encode(&mut self, video: &Video) {
        self.put_bytes(video.get_bytes());
    }
}
