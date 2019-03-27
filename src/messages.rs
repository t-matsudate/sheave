use std::{
    collections::{
        HashMap
    },
    time::{
        Duration
    }
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum MessageFormat {
    New,
    SameSource,
    TimerChange,
    Continue
}

impl From<u8> for MessageFormat {
    fn from(message_format: u8) -> Self {
        use MessageFormat::*;

        match message_format {
            0 => New,
            1 => SameSource,
            2 => TimerChange,
            3 => Continue,
            _ => panic!("Undefined message format!")
        }
    }
}

#[derive(Debug)]
pub(crate) enum BasicHeader {
    OneByte {
        message_format: MessageFormat,
        channel_id: u8
    },
    TwoBytes {
        message_format: MessageFormat,
        channel_id: u8
    },
    ThreeBytes {
        message_format: MessageFormat,
        channel_id: u16
    }
}

impl BasicHeader {
    pub(crate) const MESSAGE_HEADER_FORMAT: u8 = 0xc0;
    pub(crate) const BASIC_HEADER_TYPE: u8 = 0x3f;
    pub(crate) const LEN_ONE_BYTE: usize = 1;
    pub(crate) const LEN_TWO_BYTES: usize = 2;

    pub(crate) fn get_message_format(&self) -> MessageFormat {
        use BasicHeader::*;

        match self {
            &OneByte {
                message_format,
                channel_id: _
            }
            | &TwoBytes {
                message_format,
                channel_id: _
            }
            | &ThreeBytes {
                message_format,
                channel_id: _
            } => message_format
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum MessageType {
    Invoke = 0x14,
    Unknown
}

impl From<u8> for MessageType {
    fn from(message_type_id: u8) -> MessageType {
        use MessageType::*;

        match message_type_id {
            0x14 => Invoke,
            _ => Unknown
        }
    }
}

#[derive(Debug)]
pub(crate) enum MessageHeader {
    New {
        message_type: MessageType,
        channel_id: u32,
        message_len: usize,
        timestamp: Duration,
    },
    SameSource {
        message_type: MessageType,
        message_len: usize,
        timestamp: Duration,
    },
    TimerChange {
        timestamp: Duration,
    },
    Continue
}

impl MessageHeader {
    pub(crate) const LEN_NEW: usize = 11;
    pub(crate) const LEN_SAME_SOURCE: usize = 7;
    pub(crate) const LEN_TIMER_CHANGE: usize = 3;

    pub(crate) fn get_message_type(&self) -> Option<MessageType> {
        use MessageHeader::*;

        match self {
            &New {
                message_type,
                message_len: _,
                timestamp: _,
                channel_id: _,
            }
            | &SameSource {
                message_type,
                message_len: _,
                timestamp: _
            } => Some(message_type),
            _ => None
        }
    }

    pub(crate) fn get_message_len(&self) -> Option<usize> {
        use MessageHeader::*;

        match self {
            &New {
                message_type: _,
                message_len,
                channel_id: _,
                timestamp: _,
            }
            | &SameSource {
                message_type: _,
                message_len,
                timestamp: _,
            } => Some(message_len),
            _ => None
        }
    }

    pub(crate) fn get_timestamp(&self) -> Option<Duration> {
        use MessageHeader::*;

        match self {
            &New {
                message_type: _,
                message_len: _,
                channel_id: _,
                timestamp,
            }
            | &SameSource {
                message_type: _,
                message_len: _,
                timestamp,
            }
            | &TimerChange {
                timestamp
            } => Some(timestamp),
            _ => None
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
pub(crate) enum AmfDataType {
    Number,
    Boolean,
    String,
    Object,
    MovieClip,
    Null,
    Undefined,
    Reference,
    MixedArray,
    ObjectEnd,
    Array,
    Date,
    LongString,
    Unsupported,
    RecordSet,
    Xml,
    NamedObject,
    Amf3
}

impl From<u8> for AmfDataType {
    fn from(amf_data_type_id: u8) -> AmfDataType {
        use AmfDataType::*;
        use AmfDataType::String as AmfString;

        match amf_data_type_id {
            0x00 => Number,
            0x01 => Boolean,
            0x02 => AmfString,
            0x03 => Object,
            0x04 => MovieClip,
            0x05 => Null,
            0x06 => Undefined,
            0x07 => Reference,
            0x08 => MixedArray,
            0x09 => ObjectEnd,
            0x0a => Array,
            0x0b => Date,
            0x0c => LongString,
            0x0d => Unsupported,
            0x0e => RecordSet,
            0x0f => Xml,
            0x10 => NamedObject,
            0x11 => Amf3,
            _ => panic!("Undefined amf data type number!")
        }
    }
}

#[derive(Debug)]
pub(crate) enum AmfData {
    Number(f64),
    Boolean(bool),
    String(String),
    Object(HashMap<String, AmfData>),
    MovieClip,
    Null,
    Undefined,
    Reference,
    MixedArray(HashMap<String, AmfData>),
    ObjectEnd,
    Array(Vec<AmfData>),
    Date(f64, u16),
    LongString(String),
    Unsupported,
    RecordSet,
    Xml(String),
    NamedObject(String, HashMap<String, AmfData>),
    ToAmf3(Vec<u8>)
}

impl AmfData {
    pub(crate) const OBJECT_END_SEQUENCE: [u8; 3] = [0, 0, AmfDataType::ObjectEnd as u8];

    pub(crate) fn number(self) -> Option<f64> {
        match self {
            AmfData::Number(n) => Some(n),
            _ => None
        }
    }

    pub(crate) fn boolean(self) -> Option<bool> {
        match self {
            AmfData::Boolean(b) => Some(b),
            _ => None
        }
    }

    pub(crate) fn string(self) -> Option<String> {
        match self {
            AmfData::String(s) => Some(s),
            _ => None
        }
    }

    pub(crate) fn object(self) -> Option<HashMap<String, AmfData>> {
        match self {
            AmfData::Object(o) => Some(o),
            _ => None
        }
    }
}

#[derive(Debug)]
pub(crate) struct Argument;

#[repr(u16)]
#[derive(Debug)]
pub(crate) enum AudioCodec {
    None = 0x0001,
    Adpcm,
    Mp3 = 0x0004,
    Intel = 0x0008,
    Unused = 0x0010,
    Nerry8 = 0x0020,
    Nerry = 0x0040,
    G711a = 0x0080,
    G711u = 0x0100,
    Nerry16 = 0x0200,
    Aac = 0x0400,
    Speex = 0x0800,
    All = 0x0fff
}

impl From<u16> for AudioCodec {
    fn from(codec_flag: u16) -> AudioCodec {
        use AudioCodec::*;
        use AudioCodec::None as AudioCodecNone;

        match codec_flag {
            0x0001 => AudioCodecNone,
            0x0002 => Adpcm,
            0x0004 => Mp3,
            0x0008 => Intel,
            0x0010 => Unused,
            0x0020 => Nerry8,
            0x0040 => Nerry,
            0x0080 => G711a,
            0x0100 => G711u,
            0x0200 => Nerry16,
            0x0400 => Aac,
            0x0800 => Speex,
            0x0fff => All,
            _ => panic!("Undefined audio codec flag!")
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
pub(crate) enum VideoCodec {
    Unused = 0x01,
    Jpeg,
    Sorenson = 0x04,
    Homebrew = 0x08,
    Vp6 = 0x10,
    Vp6a = 0x20,
    Homebrewv2 = 0x40,
    H264 = 0x80,
    All = 0xff
}

impl From<u8> for VideoCodec {
    fn from(codec_flag: u8) -> VideoCodec {
        use VideoCodec::*;

        match codec_flag {
            0x01 => Unused,
            0x02 => Jpeg,
            0x04 => Sorenson,
            0x08 => Homebrew,
            0x10 => Vp6,
            0x20 => Vp6a,
            0x40 => Homebrewv2,
            0x80 => H264,
            0xff => All,
            _ => panic!("Undefined video codec flag!")
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
pub(crate) enum VideoFunction {
    ClientSeek = 0x01
}

impl From<u8> for VideoFunction {
    fn from(function_flag: u8) -> VideoFunction {
        use VideoFunction::*;

        match function_flag {
            0x01 => ClientSeek,
            _ => panic!("Undefined video function flag!")
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
pub(crate) enum ObjectEncoding {
    Amf0,
    Amf3 = 0x03
}

impl From<u8> for ObjectEncoding {
    fn from(encoding_type: u8) -> ObjectEncoding {
        use ObjectEncoding::*;

        match encoding_type {
            0x00 => Amf0,
            0x03 => Amf3,
            _ => panic!("Undefined encoding type!")
        }
    }
}

#[derive(Debug)]
pub(crate) struct CommandObject {
    fpad: Option<bool>,
    object_encoding: Option<ObjectEncoding>,
    video_function: Option<VideoFunction>,
    video_codec: Option<VideoCodec>,
    audio_codec: Option<AudioCodec>,
    app: Option<String>,
    command_type: Option<String>,
    flash_ver: Option<String>,
    swf_url: Option<String>,
    tc_url: Option<String>,
    page_url: Option<String>,
}

impl CommandObject {
    pub(crate) fn new() -> Self {
        CommandObject {
            fpad: None,
            object_encoding: None,
            video_function: None,
            video_codec: None,
            audio_codec: None,
            app: None,
            command_type: None,
            flash_ver: None,
            swf_url: None,
            tc_url: None,
            page_url: None
        }
    }

    pub(self) fn set_fpad(&mut self, fpad: Option<bool>) {
        self.fpad = fpad;
    }

    pub(self) fn set_object_encoding(&mut self, object_encoding: Option<f64>) {
        self.object_encoding = object_encoding.map(
            |object_encoding| ((object_encoding as u64) as u8).into()
        );
    }

    pub(self) fn set_video_function(&mut self, video_function: Option<f64>) {
        self.video_function = video_function.map(
            |video_function| ((video_function as u64) as u8).into()
        );
    }

    pub(self) fn set_video_codec(&mut self, video_codec: Option<f64>) {
        self.video_codec = video_codec.map(
            |video_codec| ((video_codec as u64) as u8).into()
        );
    }

    pub(self) fn set_audio_codec(&mut self, audio_codec: Option<f64>) {
        self.audio_codec = audio_codec.map(
            |audio_codec| ((audio_codec as u64) as u16).into()
        );
    }

    pub(self) fn set_app(&mut self, app: Option<String>) {
        self.app = app;
    }

    pub(self) fn set_command_type(&mut self, command_type: Option<String>) {
        self.command_type = command_type;
    }

    pub(self) fn set_flash_ver(&mut self, flash_ver: Option<String>) {
        self.flash_ver = flash_ver;
    }

    pub(self) fn set_swf_url(&mut self, swf_url: Option<String>) {
        self.swf_url = swf_url;
    }

    pub(self) fn set_tc_url(&mut self, tc_url: Option<String>) {
        self.tc_url = tc_url;
    }

    pub(self) fn set_page_url(&mut self, page_url: Option<String>) {
        self.page_url = page_url;
    }
}

impl From<HashMap<String, AmfData>> for CommandObject {
    fn from(m: HashMap<String, AmfData>) -> Self {
        let mut command_object = CommandObject::new();

        for (key, value) in m.into_iter() {
            if key == "fpad" {
                command_object.set_fpad(value.boolean());
            } else if key == "objectEncoding" {
                command_object.set_object_encoding(value.number());
            } else if key == "videoFunction" {
                command_object.set_video_function(value.number());
            } else if key == "videoCodecs" {
                command_object.set_video_codec(value.number());
            } else if key == "audioCodecs" {
                command_object.set_audio_codec(value.number());
            } else if key == "app" {
                command_object.set_app(value.string());
            } else if key == "type" {
                command_object.set_command_type(value.string());
            } else if key == "flashVer" {
                command_object.set_flash_ver(value.string());
            } else if key == "swfUrl" {
                command_object.set_swf_url(value.string());
            } else if key == "tcUrl" {
                command_object.set_tc_url(value.string());
            } else if key == "pageUrl" {
                command_object.set_page_url(value.string());
            } else {
                info!("Unknown command object: key {}, value {:?}", key, value);
            }
        }

        command_object
    }
}

#[derive(Debug)]
pub(crate) enum NetConnectionCommand {
    Connect {
        argument: Option<Argument>,
        transaction_id: u64,
        command_object: CommandObject
    }
}

#[derive(Debug)]
pub(crate) enum InvokeCommand {
    NetConnection(NetConnectionCommand)
}

#[derive(Debug)]
pub(crate) enum ChunkData {
    Invoke(InvokeCommand),
    Unknown(Vec<u8>)
}

#[derive(Debug)]
pub(crate) struct Chunk {
    basic_header: BasicHeader,
    extended_timestamp: Option<Duration>,
    message_header: MessageHeader,
    chunk_data: Option<ChunkData>
}

impl Chunk {
    pub(crate) fn new(basic_header: BasicHeader, message_header: MessageHeader, extended_timestamp: Option<Duration>, chunk_data: Option<ChunkData>) -> Self {
        Chunk {
            basic_header,
            extended_timestamp,
            message_header,
            chunk_data
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ByteBuffer {
    offset: usize,
    len: usize,
    bytes: Vec<u8>,
}

impl ByteBuffer {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        let len = bytes.len();

        ByteBuffer {
            offset: usize::default(),
            len,
            bytes
        }
    }

    pub(crate) fn offset(&mut self) -> usize {
        self.offset
    }

    pub(crate) fn offset_to(&mut self, offset: usize) {
        self.offset += offset;
    }

    pub(crate) fn len(&mut self) -> usize {
        self.len
    }

    pub(crate) fn add_len(&mut self, len: usize) {
        self.len += len;
    }

    pub(crate) fn bytes(&mut self) -> &Vec<u8> {
        &self.bytes
    }
}

pub(crate) trait GetByteBuffer {
    fn get_u8(&mut self) -> Option<u8>;
    fn get_u16_be(&mut self) -> Option<u16>;
    fn get_u24_be(&mut self) -> Option<u32>;
    fn get_u32_be(&mut self) -> Option<u32>;
    fn get_u32_le(&mut self) -> Option<u32>;
    fn get_f64(&mut self) -> Option<f64>;
    fn get_sliced_bytes(&mut self, len: usize) -> Option<Vec<u8>>;
    fn peek_byte(&mut self) -> Option<u8>;
    fn peek_bytes(&mut self, len: usize) -> Option<Vec<u8>>;
}
