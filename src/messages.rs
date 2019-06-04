use std::{
    collections::{
        HashMap
    },
    time::{
        Duration
    }
};

pub(crate) const U24_MAX: u32 = 0x00ffffff;

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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum ChunkId {
    U8(u8),
    U16(u16)
}

impl Default for ChunkId {
    fn default() -> Self {
        ChunkId::U8(u8::default())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BasicHeader {
    message_format: MessageFormat,
    chunk_id: ChunkId
}

impl BasicHeader {
    pub(crate) const MESSAGE_HEADER_FORMAT: u8 = 0xc0;
    pub(crate) const BASIC_HEADER_TYPE: u8 = 0x3f;
    pub(crate) const LEN_ONE_BYTE: usize = 1;
    pub(crate) const LEN_TWO_BYTES: usize = 2;

    pub(crate) fn new(message_format: MessageFormat, chunk_id: ChunkId) -> Self {
        BasicHeader { message_format, chunk_id }
    }

    pub(crate) fn get_message_format(&self) -> MessageFormat {
        self.message_format
    }

    pub(crate) fn get_chunk_id(&self) -> ChunkId {
        self.chunk_id
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageType {
    ChunkSize = 0x01,
    BytesRead = 0x03,
    Ping,
    ServerBandwidth,
    ClientBandwidth,
    Invoke = 0x14,
    Unknown
}

impl From<u8> for MessageType {
    fn from(message_type_id: u8) -> MessageType {
        use MessageType::*;

        match message_type_id {
            0x01 => ChunkSize,
            0x03 => BytesRead,
            0x04 => Ping,
            0x05 => ServerBandwidth,
            0x06 => ClientBandwidth,
            0x14 => Invoke,
            _ => Unknown
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MessageHeader {
    New {
        message_type: MessageType,
        message_id: u32,
        message_len: u32,
        timestamp: Duration,
    },
    SameSource {
        message_type: MessageType,
        message_len: u32,
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
                message_id: _,
            }
            | &SameSource {
                message_type,
                message_len: _,
                timestamp: _
            } => Some(message_type),
            _ => None
        }
    }

    pub(crate) fn get_message_len(&self) -> Option<u32> {
        use MessageHeader::*;

        match self {
            &New {
                message_type: _,
                message_len,
                message_id: _,
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

    pub(crate) fn set_message_len(&mut self, len: u32) -> Option<()> {
        use MessageHeader::*;

        match self {
            &mut New {
                message_type: _,
                ref mut message_len,
                message_id: _,
                timestamp: _
            } => Some(*message_len = len),
            &mut SameSource {
                message_type: _,
                ref mut message_len,
                timestamp: _
            } => Some(*message_len = len),
            _ => None
        }
    }

    pub(crate) fn get_timestamp(&self) -> Option<Duration> {
        use MessageHeader::*;

        match self {
            &New {
                message_type: _,
                message_len: _,
                message_id: _,
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

    pub(crate) fn get_message_id(&self) -> Option<u32> {
        use MessageHeader::*;

        match self {
            &New {
                message_type: _,
                message_len: _,
                message_id,
                timestamp: _,
            } => Some(message_id),
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub(crate) struct Argument;

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Default)]
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

impl From<CommandObject> for HashMap<String, AmfData> {
    fn from(command_object: CommandObject) -> Self {
        match command_object {
            CommandObject {
                fpad,
                object_encoding,
                video_function,
                video_codec,
                audio_codec,
                app,
                command_type,
                flash_ver,
                swf_url,
                tc_url,
                page_url
            } => {
                let mut m: HashMap<String, AmfData> = HashMap::new();

                app.map(|app| m.insert("app".to_string(), AmfData::String(app)));
                command_type.map(|command_type| m.insert("type".to_string(), AmfData::String(command_type)));
                flash_ver.map(|flash_ver| m.insert("flashVer".to_string(), AmfData::String(flash_ver)));
                swf_url.map(|swf_url| m.insert("swfUrl".to_string(), AmfData::String(swf_url)));
                tc_url.map(|tc_url| m.insert("tcUrl".to_string(), AmfData::String(tc_url)));
                fpad.map(|fpad| m.insert("fpad".to_string(), AmfData::Boolean(fpad)));
                audio_codec.map(|audio_codec| m.insert("audioCodecs".to_string(), AmfData::Number(f64::from_bits(audio_codec as u16 as u64))));
                video_codec.map(|video_codec| m.insert("videoCodecs".to_string(), AmfData::Number(f64::from_bits(video_codec as u8 as u64))));
                video_function.map(|video_function| m.insert("videoFunction".to_string(), AmfData::Number(f64::from_bits(video_function as u8 as u64))));
                page_url.map(|page_url| m.insert("pageUrl".to_string(), AmfData::String(page_url)));
                object_encoding.map(|object_encoding| m.insert("objectEncoding".to_string(), AmfData::Number(f64::from_bits(object_encoding as u8 as u64))));
                m
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum NetConnectionResult {
    Result,
    Error
}

impl From<String> for NetConnectionResult {
    fn from(command: String) -> Self {
        if command == "_result" {
            NetConnectionResult::Result
        } else if command == "_error" {
            NetConnectionResult::Error
        } else {
            panic!("Undefined net connection result!")
        }
    }
}

impl From<NetConnectionResult> for String {
    fn from(result: NetConnectionResult) -> Self {
        match result {
            NetConnectionResult::Result => "_result".to_string(),
            NetConnectionResult::Error => "_error".to_string()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PublishStatus {
    Start
}

impl From<String> for PublishStatus {
    fn from(publish_status: String) -> Self {
        use PublishStatus::*;

        if publish_status.starts_with("Start") {
            Start
        } else {
            panic!("Undefined publish status!")
        }
    }
}

impl From<PublishStatus> for String {
    fn from(publish_status: PublishStatus) -> Self {
        use PublishStatus::*;

        match publish_status {
            Start => "Start".to_string()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum NetStreamStatus {
    Publish(PublishStatus)
}

impl From<String> for NetStreamStatus {
    fn from(net_stream_status: String) -> Self {
        use NetStreamStatus::*;

        if net_stream_status.starts_with("Publish") {
            Publish(net_stream_status[(net_stream_status.find(".").unwrap() + 1)..].to_string().into())
        } else {
            panic!("Undefined NetStream status!")
        }
    }
}

impl From<NetStreamStatus> for String {
    fn from(net_stream_status: NetStreamStatus) -> Self {
        use NetStreamStatus::*;

        match net_stream_status {
            Publish(publish_status) => {
                let ps: String = publish_status.into();

                "Publish.".to_string() + ps.as_str()
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Status {
    NetStream(NetStreamStatus)
}

impl From<String> for Status {
    fn from(status: String) -> Self {
        use Status::*;

        if status.starts_with("NetStream") {
            NetStream(status[(status.find(".").unwrap() + 1)..].to_string().into())
        } else {
            panic!("Undefined status!")
        }
    }
}

impl From<Status> for String {
    fn from(status: Status) -> Self {
        use Status::*;

        match status {
            NetStream(net_stream_status) => {
                let nss: String = net_stream_status.into();

                "NetStream.".to_string() + nss.as_str()
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct InfoObject {
    object_encoding: Option<ObjectEncoding>,
    code: Option<Status>,
    level: Option<String>,
    details: Option<String>,
    description: Option<String>
}

impl InfoObject {
    pub(crate) fn new() -> Self {
        InfoObject {
            object_encoding: None,
            code: None,
            level: None,
            details: None,
            description: None
        }
    }

    pub(self) fn set_object_encoding(&mut self, object_encoding: Option<f64>) {
        self.object_encoding = object_encoding.map(|object_encoding| (object_encoding as u64 as u8).into());
    }

    pub(self) fn set_code(&mut self, code: Option<String>) {
        self.code = code.map(|code| code.into());
    }

    pub(self) fn set_level(&mut self, level: Option<String>) {
        self.level = level;
    }

    pub(self) fn set_details(&mut self, details: Option<String>) {
        self.details = details;
    }

    pub(self) fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }
}

impl From<HashMap<String, AmfData>> for InfoObject {
    fn from(m: HashMap<String, AmfData>) -> Self {
        let mut info_object = InfoObject::new();

        for (key, value) in m {
            if key == "objectEncoding" {
                info_object.set_object_encoding(value.number());
            } else if key == "code" {
                info_object.set_code(value.string());
            } else if key == "level" {
                info_object.set_level(value.string());
            } else if key == "details" {
                info_object.set_details(value.string());
            } else if key == "description" {
                info_object.set_description(value.string());
            } else {
                println!("Unknown info object: key {}, value {:?}", key, value);
            }
        }

        info_object
    }
}

impl From<InfoObject> for HashMap<String, AmfData> {
    fn from(info_object: InfoObject) -> Self {
        match info_object {
            InfoObject {
                object_encoding,
                code,
                level,
                details,
                description
            } => {
                let mut m: HashMap<String, AmfData> = HashMap::new();

                object_encoding.map(|object_encoding| m.insert("objectEncoding".to_string(), AmfData::Number(object_encoding as u8 as u64 as f64)));
                code.map(|code| m.insert("code".to_string(), AmfData::String(code.into())));
                level.map(|level| m.insert("level".to_string(), AmfData::String(level)));
                details.map(|details| m.insert("details".to_string(), AmfData::String(details)));
                description.map(|description| m.insert("description".to_string(), AmfData::String(description)));
                m
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum NetConnectionCommand {
    Connect {
        argument: Option<Argument>,
        transaction_id: u64,
        command_object: CommandObject
    },
    ConnectResult {
        result: NetConnectionResult,
        transaction_id: u64,
        properties: HashMap<String, AmfData>,
        information: HashMap<String, AmfData>
    },
    ReleaseStream {
        transaction_id: u64,
        play_path: String
    },
    ReleaseStreamResult {
        result: NetConnectionResult,
        transaction_id: u64
    },
    CreateStream {
        transaction_id: u64
    },
    CreateStreamResult {
        result: NetConnectionResult,
        message_id: u32,
        transaction_id: u64
    }
}

#[derive(Debug, Clone)]
pub(crate) enum FcPublishCommand {
    FcPublish {
        transaction_id: u64,
        play_path: String
    },
    OnFcPublish
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PlayType {
    Live,
    Record,
    Append
}

impl From<String> for PlayType {
    fn from(s: String) -> Self {
        use PlayType::*;

        if s == "live" {
            Live
        } else if s == "record" {
            Record
        } else if s == "append" {
            Append
        } else {
            panic!("Undefined publishing type!")
        }
    }
}

impl From<PlayType> for String {
    fn from(play_type: PlayType) -> Self {
        use PlayType::*;

        match play_type {
            Live => "live".to_string(),
            Record => "record".to_string(),
            Append => "append".to_string()
        }
    }
}

impl Default for PlayType {
    fn default() -> Self {
        PlayType::Live
    }
}

#[derive(Debug, Clone)]
pub(crate) enum NetStreamCommand {
    Publish {
        transaction_id: u64,
        play_path: String,
        play_type: PlayType
    },
    OnStatus {
        transaction_id: u64,
        info_object: InfoObject
    }
}

#[derive(Debug, Clone)]
pub(crate) enum InvokeCommand {
    NetConnection(NetConnectionCommand),
    NetStream(NetStreamCommand),
    FcPublish(FcPublishCommand),
    Unknown(Vec<u8>)
}

impl InvokeCommand {
    pub(crate) fn is_connect(&self) -> bool {
        match self {
            &InvokeCommand::NetConnection(
                NetConnectionCommand::Connect {
                    argument: _,
                    transaction_id: _,
                    command_object: _
                }
            ) => true,
            _ => false
        }
    }

    pub(crate) fn is_release_stream(&self) -> bool {
        match self {
            &InvokeCommand::NetConnection(
                NetConnectionCommand::ReleaseStream {
                    transaction_id: _,
                    play_path: _
                }
            ) => true,
            _ => false
        }
    }

    pub(crate) fn is_create_stream(&self) -> bool {
        match self {
            &InvokeCommand::NetConnection(
                NetConnectionCommand::CreateStream {
                    transaction_id: _
                }
            ) => true,
            _ => false
        }
    }

    pub(crate) fn net_connection(&self) -> Option<&NetConnectionCommand> {
        match self {
            &InvokeCommand::NetConnection(ref net_connection_command) => Some(net_connection_command),
            _ => None
        }
    }

    pub(crate) fn is_fc_publish(&self) -> bool {
        match self {
            &InvokeCommand::FcPublish(_) => true,
            _ => false
        }
    }

    pub(crate) fn fc_publish(&self) -> Option<&FcPublishCommand> {
        match self {
            &InvokeCommand::FcPublish(ref fc_publish_command) => Some(fc_publish_command),
            _ => None
        }
    }

    pub(crate) fn is_publish(&self) -> bool {
        match self {
            &InvokeCommand::NetStream(
                NetStreamCommand::Publish {
                    transaction_id: _,
                    play_path: _,
                    play_type: _
                }
            ) => true,
            _ => false
        }
    }

    pub(crate) fn net_stream(&self) -> Option<&NetStreamCommand> {
        match self {
            &InvokeCommand::NetStream(ref net_stream_command) => Some(net_stream_command),
            _ => None
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum LimitType {
    Hard,
    Soft,
    Dynamic
}

impl From<u8> for LimitType {
    fn from(limit_type: u8) -> Self {
        use LimitType::*;

        match limit_type {
            0 => Hard,
            1 => Soft,
            2 => Dynamic,
            _ => panic!("Undefined limit type!")
        }
    }
}

impl Default for LimitType {
    fn default() -> Self {
        LimitType::Hard
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum PingType {
    StreamBegin
}

impl From<u8> for PingType {
    fn from(ping_type: u8) -> Self {
        use PingType::*;

        match ping_type {
            0 => StreamBegin,
            _ => panic!("Undefined ping type!")
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PingData {
    StreamBegin(u32)
}

#[derive(Debug, Clone)]
pub(crate) enum ChunkData {
    ChunkSize(u32),
    BytesRead(u32),
    Ping(PingData),
    ServerBandwidth(u32),
    ClientBandwidth(u32, LimitType),
    Invoke(InvokeCommand),
    Unknown(Vec<u8>)
}

impl ChunkData {
    pub(crate) fn chunk_size(&self) -> Option<u32> {
        match self {
            &ChunkData::ChunkSize(chunk_size) => Some(chunk_size),
            _ => None
        }
    }

    pub(crate) fn bytes_read(&self) -> Option<u32> {
        match self {
            &ChunkData::BytesRead(bytes_read) => Some(bytes_read),
            _ => None
        }
    }

    pub(crate) fn invoke(&self) -> Option<&InvokeCommand> {
        match self {
            &ChunkData::Invoke(ref invoke_command) => Some(invoke_command),
            _ => None
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Chunk {
    basic_header: BasicHeader,
    extended_timestamp: Option<Duration>,
    message_header: MessageHeader,
    chunk_data: Option<ChunkData>
}

impl Chunk {
    pub(crate) fn new(basic_header: BasicHeader, extended_timestamp: Option<Duration>, message_header: MessageHeader, chunk_data: Option<ChunkData>) -> Self {
        Chunk {
            basic_header,
            extended_timestamp,
            message_header,
            chunk_data
        }
    }

    pub(crate) fn get_basic_header(&self) -> BasicHeader {
        self.basic_header
    }

    pub(crate) fn get_extended_timestamp(&self) -> Option<Duration> {
        self.extended_timestamp
    }

    pub(crate) fn get_message_header(&self) -> MessageHeader {
        self.message_header
    }

    pub(crate) fn get_chunk_data(&self) -> &Option<ChunkData> {
        &self.chunk_data
    }
}

#[derive(Debug, Default)]
pub(crate) struct ByteBuffer {
    offset: usize,
    len: usize,
    bytes: Vec<u8>
}

impl ByteBuffer {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        ByteBuffer {
            offset: 0,
            len: bytes.len(),
            bytes
        }
    }

    pub(crate) fn clear(&mut self) {
        self.offset = 0;
        self.len = 0;
        self.bytes = Vec::new();
    }

    pub(crate) fn offset(&self) -> usize {
        self.offset
    }

    pub(crate) fn offset_to(&mut self, offset: usize) {
        self.offset += offset;
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn add_len(&mut self, len: usize) {
        self.len += len;
    }

    pub(crate) fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub(crate) fn bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }
}

pub(crate) trait GetByteBuffer {
    fn get_u8(&mut self) -> Option<u8>;
    fn get_u16_be(&mut self) -> Option<u16>;
    fn get_u16_le(&mut self) -> Option<u16>;
    fn get_u24_be(&mut self) -> Option<u32>;
    fn get_u32_be(&mut self) -> Option<u32>;
    fn get_u32_le(&mut self) -> Option<u32>;
    fn get_f64(&mut self) -> Option<f64>;
    fn get_sliced_bytes(&mut self, len: usize) -> Option<Vec<u8>>;
    fn peek_byte(&self) -> Option<u8>;
    fn peek_bytes(&self, len: usize) -> Option<Vec<u8>>;
}

pub(crate) trait PutByteBuffer {
    fn put_u8(&mut self, byte: u8);
    fn put_u16_be(&mut self, byte: u16);
    fn put_u16_le(&mut self, byte: u16);
    fn put_u24_be(&mut self, byte: u32);
    fn put_u32_be(&mut self, byte: u32);
    fn put_u32_le(&mut self, byte: u32);
    fn put_f64(&mut self, byte: f64);
    fn put_bytes(&mut self, bytes: Vec<u8>);
}
