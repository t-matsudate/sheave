use std::io::{
    Error as IOError,
    Result as IOResult
};
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::Video
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Key = 1,
    Inter,
    Disposable,
    Generated,
    Info,
    Other = 15,
}

impl From<u8> for FrameType {
    fn from(frame_type: u8) -> Self {
        use FrameType::*;

        match frame_type {
            1 => Key,
            2 => Inter,
            3 => Disposable,
            4 => Generated,
            5 => Info,
            _ => Other
        }
    }
}

impl From<FrameType> for u8 {
    fn from(frame_type: FrameType) -> Self {
        frame_type as u8
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    H263 = 2,
    Screen,
    Vp6,
    Vp6a,
    Screen2,
    Avc,
    Other = 15
}

impl Codec {
    pub fn is_avc(&self) -> bool {
        match *self {
            Codec::Avc => true,
            _ => false
        }
    }
}

impl From<u8> for Codec {
    fn from(codec: u8) -> Self {
        use Codec::*;

        match codec {
            2 => H263,
            3 => Screen,
            4 => Vp6,
            5 => Vp6a,
            6 => Screen2,
            7 => Avc,
            _ => Other
        }
    }
}

impl From<Codec> for u8 {
    fn from(codec: Codec) -> Self {
        codec as u8
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvcPacketType {
    SequenceHeader,
    Nalu,
    SequenceEnd,
    Other = 0xff
}

impl From<u8> for AvcPacketType {
    fn from(avc_packet_type: u8) -> Self {
        use AvcPacketType::*;

        match avc_packet_type {
            0 => SequenceHeader,
            1 => Nalu,
            2 => SequenceEnd,
            _ => Other
        }
    }
}

impl From<AvcPacketType> for u8 {
    fn from(avc_packet_type: AvcPacketType) -> Self {
        avc_packet_type as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoTagHeader {
    frame_type: FrameType,
    codec: Codec,
    avc_packet_type: Option<AvcPacketType>,
    composition_time: Option<i32>,
}

impl VideoTagHeader {
    pub fn new(frame_type: FrameType, codec: Codec, avc_packet_type: Option<AvcPacketType>, composition_time: Option<i32>) -> Self {
        Self { frame_type, codec, avc_packet_type, composition_time }
    }
}

impl Decoder<VideoTagHeader> for ByteBuffer {
    fn decode(&mut self) -> IOResult<VideoTagHeader> {
        let byte = self.get_u8()?;
        let frame_type: FrameType = (byte >> 4).into();
        let codec: Codec = (byte & 0x0f).into();
        let is_avc = codec.is_avc();
        let avc_packet_type: Option<AvcPacketType> = if !is_avc {
            None
        } else {
            let byte = self.get_u8()?;
            Some(byte.into())
        };
        let composition_time = if !is_avc {
            None
        } else {
            let bytes = self.get_i24_be()?;
            Some(bytes)
        };

        Ok(VideoTagHeader { frame_type, codec, avc_packet_type, composition_time })
    }
}

impl Encoder<VideoTagHeader> for ByteBuffer {
    fn encode(&mut self, video_tag_header: &VideoTagHeader) {
        let mut byte = u8::from(video_tag_header.frame_type) << 4;
        byte |= u8::from(video_tag_header.codec);
        self.put_u8(byte);

        if let Some(avc_packet_type) = video_tag_header.avc_packet_type {
            self.put_u8(avc_packet_type.into());
            self.put_i24_be(video_tag_header.composition_time.unwrap());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoTag {
    header: VideoTagHeader,
    body: Vec<u8>
}

impl VideoTag {
    pub fn new(header: VideoTagHeader, body: Vec<u8>) -> Self {
        Self { header, body }
    }
}

impl Decoder<VideoTag> for ByteBuffer {
    fn decode(&mut self) -> IOResult<VideoTag> {
        let header: VideoTagHeader = self.decode()?;
        let remained = self.remained();
        let body = self.get_bytes(remained)?.to_vec();

        Ok(VideoTag { header, body })
    }
}

impl Encoder<VideoTag> for ByteBuffer {
    fn encode(&mut self, video_tag: &VideoTag) {
        self.encode(&video_tag.header);
        self.put_bytes(&video_tag.body);
    }
}

impl TryFrom<Video> for VideoTag {
    type Error = IOError;

    fn try_from(video: Video) -> IOResult<Self> {
        let mut buffer: ByteBuffer = Vec::<u8>::from(video).into();
        Decoder::<Self>::decode(&mut buffer)
    }
}

impl TryFrom<VideoTag> for Video {
    type Error = IOError;

    fn try_from(video_tag: VideoTag) -> IOResult<Self> {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&video_tag);
        Ok(Self::new(buffer.into()))
    }
}

#[cfg(test)]
mod tests {
    use rand::{
        Fill,
        thread_rng
    };
    use super::*;

    #[test]
    fn decode_video_tag() {
        let mut buffer = ByteBuffer::default();
        let mut byte = u8::from(FrameType::Disposable) << 4;
        byte |= u8::from(Codec::H263);
        buffer.put_u8(byte);
        let mut data: [u8; 128] = [0; 128];
        data.try_fill(&mut thread_rng()).unwrap();
        buffer.put_bytes(&data);
        assert!(Decoder::<VideoTag>::decode(&mut buffer).is_ok())
    }

    #[test]
    fn encode_video_tag() {
        let mut buffer = ByteBuffer::default();
        let mut expected_data: [u8; 128] = [0; 128];
        expected_data.try_fill(&mut thread_rng()).unwrap();
        let expected = VideoTag::new(
            VideoTagHeader::new(
                FrameType::Disposable,
                Codec::H263,
                None,
                None
            ),
            expected_data.to_vec()
        );
        buffer.encode(&expected);

        let byte = buffer.get_u8().unwrap();
        assert_eq!(FrameType::Disposable as u8, byte >> 4);
        assert_eq!(Codec::H263 as u8, byte & 0x0f);

        let actual_data: Vec<u8> = buffer.into();
        assert_eq!(expected_data.as_slice(), actual_data)
    }
}
