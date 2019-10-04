use std::{
    time::{
        Duration,
        SystemTime
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
    fn real_len(&self) -> u32 {
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
    fn real_len(&self) -> u32 {
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
    fn real_len(&self) -> u32 {
        4 + self.filter_name.len() as u32
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum FilterParams {
    Encryption([u8; 16]),
    SelectiveEncryption(bool, Option<[u8; 16]>)
}

impl FilterParams {
    fn real_len(&self) -> u32 {
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
pub(crate) struct EncryptionTag {
    filter_params: FilterParams,
    encryption_tag_header: EncryptionTagHeader
}

impl EncryptionTag {
    fn real_len(&self) -> u32 {
        self.filter_params.real_len() + self.encryption_tag_header.real_len()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AudioTag {
    encryption_tag: Option<EncryptionTag>,
    audio_tag_header: AudioTagHeader,
    bytes: Vec<u8>
}

impl AudioTag {
    fn real_len(&self) -> u32 {
        let mut real_len = self.audio_tag_header.real_len() + self.bytes.len() as u32;

        if let &Some(ref encryption_tag) = &self.encryption_tag {
            real_len += encryption_tag.real_len();
        }

        real_len
    }
}

impl From<Vec<u8>> for AudioTag {
    fn from(mut bytes: Vec<u8>) -> AudioTag {
        let byte_audio_tag_header = bytes[0];
        let sound_format: SoundFormat = ((byte_audio_tag_header & 0xf0) >> 4).into();
        let sound_rate: SoundRate = ((byte_audio_tag_header & 0x0c) >> 2).into();
        let sound_size: SoundSize = ((byte_audio_tag_header & 0x02) >> 1).into();
        let sound_type: SoundType = (byte_audio_tag_header & 0x01).into();

        bytes.remove(0);

        let aac_packet_type: Option<AacPacketType> = if let SoundFormat::Aac = sound_format {
            let aac_packet_type_id = bytes[0];

            bytes.remove(0);
            Some(aac_packet_type_id.into())
        } else {
            None
        };
        let audio_tag_header = AudioTagHeader {
            sound_format,
            sound_rate,
            sound_size,
            sound_type,
            aac_packet_type
        };

        AudioTag {
            encryption_tag: None,
            audio_tag_header,
            bytes
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VideoTag {
    encryption_tag: Option<EncryptionTag>,
    video_tag_header: VideoTagHeader,
    bytes: Vec<u8>
}

impl VideoTag {
    fn real_len(&self) -> u32 {
        let mut real_len = self.video_tag_header.real_len() + self.bytes.len() as u32;

        if let &Some(ref encryption_tag) = &self.encryption_tag {
            real_len += encryption_tag.real_len();
        }

        real_len
    }
}

impl From<Vec<u8>> for VideoTag {
    fn from(mut bytes: Vec<u8>) -> Self {
        let byte_video_tag_header = bytes[0];
        let frame_type: FrameType = ((byte_video_tag_header & 0xf0) >> 4).into();
        let codec: Codec = (byte_video_tag_header & 0x0f).into();

        bytes.remove(0);

        let avc_packet_type: Option<AvcPacketType> = if let Codec::Avc = codec {
            let avc_packet_type_id = bytes[0];

            bytes.remove(0);
            Some(avc_packet_type_id.into())
        } else {
            None
        };
        let composition_time = if let Codec::Avc = codec {
            if let &Some(AvcPacketType::Nalu) = &avc_packet_type {
                let bytes_composition_time = &bytes[..3];
                let mut tmp: [u8; 4] = [0; 4];

                for i in 0..bytes_composition_time.len() {
                    tmp[i + 1] = bytes_composition_time[i];
                }

                bytes = bytes[3..].to_vec();
                Some(Duration::from_millis(u32::from_be_bytes(tmp) as u64))
            } else {
                bytes = bytes[3..].to_vec();
                Some(Duration::default())
            }
        } else {
            None
        };
        let video_tag_header = VideoTagHeader {
            frame_type,
            codec,
            avc_packet_type,
            composition_time
        };

        VideoTag {
            encryption_tag: None,
            video_tag_header,
            bytes
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DataTag {
    encryption_tag: Option<EncryptionTag>,
    bytes: Vec<u8>
}

impl DataTag {
    fn real_len(&self) -> u32 {
        let mut real_len = self.bytes.len() as u32;

        if let &Some(ref encryption_tag) = &self.encryption_tag {
            real_len += encryption_tag.real_len();
        }

        real_len
    }
}

impl From<MetaData> for DataTag {
    fn from(meta_data: MetaData) -> Self {
        let mut buffer = ByteBuffer::new(Vec::new());

        buffer.encode_amf_string("onMetaData".to_string());
        buffer.encode_amf_mixed_array(meta_data.into());

        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(buffer.bytes().as_slice());

        DataTag {
            encryption_tag: None,
            bytes
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum FlvData {
    Audio(AudioTag),
    Video(VideoTag),
    Data(DataTag)
}

impl FlvData {
    fn has_encryption_tag(&self) -> bool {
        match self {
            &FlvData::Audio(ref audio_tag) => audio_tag.encryption_tag.is_some(),
            &FlvData::Video(ref video_tag) => video_tag.encryption_tag.is_some(),
            &FlvData::Data(ref data_tag) => data_tag.encryption_tag.is_some()
        }
    }

    fn real_len(&self) -> u32 {
        match self {
            &FlvData::Audio(ref audio_tag) => audio_tag.real_len(),
            &FlvData::Video(ref video_tag) => video_tag.real_len(),
            &FlvData::Data(ref data_tag) => data_tag.real_len()
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FlvTag {
    is_filtered: bool,
    tag_type: TagType,
    data_size: u32,
    stream_id: u32,
    timestamp: Duration,
    data: FlvData
}

impl FlvTag {
    fn real_len(&self) -> u32 {
        11 + self.data.real_len()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FlvBody {
    previous_size: u32,
    flv_tag: FlvTag
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Flv {
    flv_header: FlvHeader,
    created: Duration,
    body: Vec<FlvBody>
}

impl Flv {
    pub(crate) fn append_meta_data(&mut self, meta_data: MetaData) {
        let previous_size = self.body.last().map_or(
            0,
            |last_flv_body| last_flv_body.previous_size + last_flv_body.flv_tag.real_len()
        );
        let data = FlvData::Data(meta_data.into());
        let is_filtered = data.has_encryption_tag();
        let data_size = data.real_len();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap() - self.created;

        self.body.push(
            FlvBody {
                previous_size,
                flv_tag: FlvTag {
                    is_filtered,
                    tag_type: TagType::ScriptData,
                    data_size,
                    stream_id: 0,
                    timestamp,
                    data
                }
            }
        );
    }

    pub(crate) fn append_audio(&mut self, bytes: Vec<u8>) {
        let previous_size = self.body.last().map_or(
            0,
            |last_flv_body| last_flv_body.previous_size + last_flv_body.flv_tag.real_len()
        );
        let data = FlvData::Audio(bytes.into());
        let is_filtered = data.has_encryption_tag();
        let data_size = data.real_len();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap() - self.created;

        self.body.push(
            FlvBody {
                previous_size,
                flv_tag: FlvTag {
                    is_filtered,
                    tag_type: TagType::Audio,
                    data_size,
                    stream_id: 0,
                    timestamp,
                    data
                }
            }
        );
        self.flv_header.has_audio = true;
    }

    pub(crate) fn append_video(&mut self, bytes: Vec<u8>) {
        let previous_size = self.body.last().map_or(
            0,
            |last_flv_body| last_flv_body.previous_size + last_flv_body.flv_tag.real_len()
        );
        let data = FlvData::Video(bytes.into());
        let is_filtered = data.has_encryption_tag();
        let data_size = data.real_len();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap() - self.created;

        self.body.push(
            FlvBody {
                previous_size,
                flv_tag: FlvTag {
                    is_filtered,
                    tag_type: TagType::Video,
                    data_size,
                    stream_id: 0,
                    timestamp,
                    data
                }
            }
        );
        self.flv_header.has_video = true;
    }
}
