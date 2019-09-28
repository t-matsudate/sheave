use std::{
    time::{
        Duration
    }
};
use crate::{
    encoders::*,
    messages::{
        ByteBuffer,
        MetaData
    }
};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FlvHeader {
    has_audio: bool,
    has_video: bool,
    version: u8,
    offset: u32
}

impl FlvHeader {
    const SIGNATURE: &'static str = &"FLV";
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum TagType {
    Audio = 8,
    Video = 9,
    ScriptData = 18
}

impl From<u8> for TagType {
    fn from(tag_type_id: u8) -> Self {
        use TagType::*;

        match tag_type_id {
            8 => Audio,
            9 => Video,
            18 => ScriptData,
            _ => panic!("Undefined tag type id!")
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum SoundFormat {
    LinearNe,
    Adpcm,
    Mp3,
    LinearLe,
    NellymoserSixteen,
    NellymoserEight,
    Nellymoser,
    G711A,
    G711mu,
    Aac = 10,
    Speex,
    Mp3Eight = 14,
    Other
}

impl From<u8> for SoundFormat {
    fn from(sound_format_id: u8) -> Self {
        use SoundFormat::*;

        match sound_format_id {
            0 => LinearNe,
            1 => Adpcm,
            2 => Mp3,
            3 => LinearLe,
            4 => NellymoserSixteen,
            5 => NellymoserEight,
            6 => Nellymoser,
            7 => G711A,
            8 => G711mu,
            10 => Aac,
            11 => Speex,
            14 => Mp3Eight,
            15 => Other,
            _ => panic!("Undefined sound format id!")
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum SoundRate {
    FivePointFive,
    Eleven,
    TwentyTwo,
    FortyFour
}

impl From<u8> for SoundRate {
    fn from(sound_rate_id: u8) -> Self {
        use SoundRate::*;

        match sound_rate_id {
            0 => FivePointFive,
            1 => Eleven,
            2 => TwentyTwo,
            3 => FortyFour,
            _ => panic!("Undefined sound rate id!")
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum SoundSize {
    Eight,
    Sixteen
}

impl From<u8> for SoundSize {
    fn from(sound_size_id: u8) -> Self {
        use SoundSize::*;

        match sound_size_id {
            0 => Eight,
            1 => Sixteen,
            _ => panic!("Undefined sound size!")
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum SoundType {
    Mono,
    Stereo
}

impl From<u8> for SoundType {
    fn from(sound_type_id: u8) -> Self {
        use SoundType::*;

        match sound_type_id {
            0 => Mono,
            1 => Stereo,
            _ => panic!("Undefined sound type id!")
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum AacPacketType {
    SequenceHeader,
    Raw
}

impl From<u8> for AacPacketType {
    fn from(aac_packet_type_id: u8) -> Self {
        use AacPacketType::*;

        match aac_packet_type_id {
            0 => SequenceHeader,
            1 => Raw,
            _ => panic!("Undefined AAC packet type!")
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AudioTagHeader {
    sound_format: SoundFormat,
    sound_rate: SoundRate,
    sound_size: SoundSize,
    sound_type: SoundType,
    aac_packet_type: Option<AacPacketType>
}

impl AudioTagHeader {
    fn size(&self) -> u32 {
        let aac_packet_type_size = if self.aac_packet_type.is_some() {
            1
        } else {
            0
        };

        1 + aac_packet_type_size
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum FrameType {
    Key = 1,
    Inter,
    DisposableInter,
    GeneratedKey,
    VideoInfo
}

impl From<u8> for FrameType {
    fn from(frame_type_id: u8) -> Self {
        use FrameType::*;

        match frame_type_id {
            1 => Key,
            2 => Inter,
            3 => DisposableInter,
            4 => GeneratedKey,
            5 => VideoInfo,
            _ => panic!("Undefined frame type id!")
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum Codec {
    SorensonH263 = 2,
    ScreenVideo,
    On2Vp6,
    On2Vp6a,
    ScreenVideo2,
    Avc
}

impl From<u8> for Codec {
    fn from(codec_id: u8) -> Self {
        use Codec::*;

        match codec_id {
            2 => SorensonH263,
            3 => ScreenVideo,
            4 => On2Vp6,
            5 => On2Vp6a,
            6 => ScreenVideo2,
            7 => Avc,
            _ => panic!("Undefined codec id!")
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum AvcPacketType {
    SequenceHeader,
    Nalu,
    EndOfSequence
}

impl From<u8> for AvcPacketType {
    fn from(avc_packet_type_id: u8) -> Self {
        use AvcPacketType::*;

        match avc_packet_type_id {
            0 => SequenceHeader,
            1 => Nalu,
            2 => EndOfSequence,
            _ => panic!("Undefined avc packet type id!")
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VideoTagHeader {
    frame_type: FrameType,
    codec: Codec,
    avc_packet_type: Option<AvcPacketType>,
    composition_time: Option<Duration>
}

impl VideoTagHeader {
    fn size(&self) -> u32 {
        let avc_packet_type_size = if self.avc_packet_type.is_some() {
            1
        } else {
            0
        };
        let composition_time_size = if self.composition_time.is_some() {
            3
        } else {
            0
        };

        1 + avc_packet_type_size + composition_time_size
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EncryptionTagHeader {
    filters_count: u8,
    length: u32,
    filter_name: String
}

impl EncryptionTagHeader {
    fn size(&self) -> u32 {
        4 + self.filter_name.len() as u32
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum FilterParams {
    Encryption([u8; 16]),
    SelectiveEncryption(bool, Option<[u8; 16]>)
}

impl FilterParams {
    fn size(&self) -> u32 {
        match self {
            &FilterParams::Encryption(_) => 16,
            &FilterParams::SelectiveEncryption(is_encrypted, _) => if is_encrypted {
                24
            } else {
                8
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FlvTag {
    is_filter: bool,
    tag_type: TagType,
    data_size: u32,
    stream_id: u32,
    timestamp: Duration,
    audio_tag_header: Option<AudioTagHeader>,
    video_tag_header: Option<VideoTagHeader>,
    encryption_tag_header: Option<EncryptionTagHeader>,
    filter_params: Option<FilterParams>,
    data: Vec<u8>
}

impl FlvTag {
    fn new(tag_type: TagType) -> Self {
        FlvTag {
            is_filter: false,
            tag_type,
            data_size: 0,
            stream_id: 0,
            timestamp: Duration::new(0, 0),
            audio_tag_header: None,
            video_tag_header: None,
            encryption_tag_header: None,
            filter_params: None,
            data: Vec::new()
        }
    }

    fn size(&self) -> u32 {
        let audio_tag_header_size = self.audio_tag_header.as_ref().map_or(
            0,
            |audio_tag_header| audio_tag_header.size()
        );
        let video_tag_header_size = self.video_tag_header.as_ref().map_or(
            0,
            |video_tag_header| video_tag_header.size()
        );
        let encryption_tag_header_size = self.encryption_tag_header.as_ref().map_or(
            0,
            |encryption_tag_header| encryption_tag_header.size()
        );
        let filter_params_size = self.filter_params.as_ref().map_or(
            0,
            |filter_params| filter_params.size()
        );

        11 + audio_tag_header_size + video_tag_header_size + encryption_tag_header_size + filter_params_size + self.data.len() as u32
    }
}

impl From<MetaData> for FlvTag {
    fn from(meta_data: MetaData) -> Self {
        let mut buffer = ByteBuffer::new(Vec::new());

        buffer.encode_amf_mixed_array(meta_data.into());

        let mut flv_tag = FlvTag::new(TagType::ScriptData);

        flv_tag.data.extend_from_slice(buffer.bytes().as_slice());
        flv_tag.data_size = buffer.bytes().len() as u32;
        flv_tag
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FlvBody {
    previous_size: u32,
    tag: FlvTag
}

impl FlvBody {
    fn new(previous_size: u32, tag: FlvTag) -> Self {
        FlvBody {
            previous_size,
            tag
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Flv {
    header: FlvHeader,
    body: Vec<FlvBody>
}

impl Flv {
    pub(crate) fn append_meta_data(&mut self, meta_data: MetaData) {
        let previous_size = self.body.last().map_or(
            0,
            |last_flv_body| last_flv_body.previous_size + last_flv_body.tag.size()
        );

        self.body.push(FlvBody::new(previous_size, meta_data.into()));
    }
}
