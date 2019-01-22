use std::{
    collections::{
        HashMap
    },
    marker::{
        PhantomData
    },
    time::{
        Duration,
    }
};

#[derive(Debug)]
enum BasicHeader {
    U8(u8),
    U16(u16),
    U24(u32)
}

#[repr(u8)]
#[derive(Debug)]
enum ChunkType {
    ChunkSize = 0x01,
    Abort,
    BytesRead,
    Ping,
    ServerBandwidth,
    ClientBandwidth,

    Audio = 0x08,
    Video,
    FlexStreamSend = 0x0f,
    FlexSharedObject,
    FlexMessage,
    Notify,
    SharedObject,
    Invoke,
    Aggregate,
    Unknown
}

#[derive(Debug)]
enum ChunkMessageHeader {
    Type0 {
        timestamp: Duration,
        message_length: usize,
        chunk_type: ChunkType,
        stream_id: u32
    },
    Type1 {
        timestamp: Duration,
        message_length: usize,
        chunk_type: ChunkType
    },
    Type2 {
        timestamp: Duration
    },
    Type3
}

#[repr(u8)]
#[derive(Debug)]
enum LimitType {
    Hard,
    Soft,
    Dynamic
}

#[derive(Debug)]
struct RtmpMessageHeader {
    chunk_type: ChunkType,
    payload: usize,
    timestamp: Duration,
    stream_id: u32
}

#[derive(Debug)]
enum PingData {
    StreamBegin,
    StreamPlayBufferClear,
    StreamDry,
    ClientBuffer {
        stream_id: u32,
        buffer_length: usize
    },
    RecordedStream,
    PingClient {
        data: u32
    },
    PongServer {
        data: u32
    },
    PingSwfVerify {
        data: Vec<u8>
    },
    PongSwfVerify {
        data: Vec<u8>
    },
    BufferEmpty,
    BufferFull
}

#[derive(Debug)]
struct AggregateBody {
    timestamp: Duration,
    size: usize,
    aggregate_type: u8
}

#[repr(u8)]
#[derive(Debug)]
enum Method {
    Connect,
    Call,
    Close,
    CreateStream,
    Play,
    Play2,
    DeleteStream,
    CloseStream,
    ReceiveAudio,
    ReceiveVideo,
    Publish,
    Seek,
    Pause,
    Result,
    Error,
    OnStatus,
    CheckBw,
    ReleaseStream,
    FCSubscribe,
    FCPublish,
    GetStreamLength,
    OnBwDone
}

#[derive(Debug)]
enum AmfData {
    Number,
    Boolean,
    String(String),
    Object,
    MovieClip,
    Null,
    Undefined,
    Reference,
    MixedArray(HashMap<String, String>),
    EndOfObject,
    Array(Vec<u8>),
    Date,
    LongString(String),
    Unsupported,
    RecordSet,
    Xml,
    ClassObject,
    Amf3Object
}

#[repr(u8)]
#[derive(Debug)]
enum Params {
    Skip,
    Null,
    Boolean,
    Number,
    String,
    Date,
    Array,
    Map,
    Xml,
    Object,
    Bytearray,
    Reference,
    VecInt,
    VecUInt,
    VecNumber,
    VecObject
}

#[derive(Debug)]
enum DataFrameMethod {
    OnCue,
    OnMeta
}

#[derive(Debug)]
enum NotifyCommand {
    OnMetaData(AMFData, Params),
    SetDataFrame(DataFrameMethod, Params)
}

#[repr(u8)]
#[derive(Debug)]
enum EventType {
    ServerConnect = 0x01,
    ServerDisconnect,
    ServerSetAttribute,
    ClientUpdataData,
    ClientUpdateAttribute,
    SendMessage,
    ClientStatus,
    ClietnClearData,
    ClientDeleteData,
    DeleteAttribute,
    ClientInitialData
}

#[derive(Debug)]
struct EventData {
    key: String,
    value: AmfData
}

#[derive(Debug)]
struct SharedObjectEvent {
    event_type: EventType,
    data: EventData
}

#[repr(u8)]
#[derive(Debug)]
enum AMF3Data {
    Undefined,
    Null,
    False,
    True,
    I32,
    Number,
    String,
    XmlDocument,
    Date,
    Array,
    Object,
    Xml,
    ByteArray,
    VecI32,
    VecU32,
    VecNumber,
    VecObject,
    Dictionary
}

#[repr(u8)]
#[derive(Debug)]
enum AMF3Object {
    Property,
    Externalizable,
    Value,
    Proxy
}

#[repr(u16)]
#[derive(Debug)]
enum AudioCodecs {
    None = 0x0001,
    Adpcm,
    Mp3 = 0x0004,
    Intel = 0x0008,
    Unused = 0x0010,
    Nelly8 = 0x0020,
    Nelly = 0x0040,
    G711a = 0x0080,
    G711u = 0x0100,
    Nelly16 = 0x0200,
    Aac = 0x0400,
    Speex = 0x0800,
    All = 0x0fff
}

#[repr(u8)]
#[derive(Debug)]
enum VideoCodecs {
    Unused = 0x01,
    Jpeg,
    Sorenson = 0x04,
    HomeBrewV1 = 0x08,
    Vp6 = 0x0010,
    Vp6Alpha = 0x20,
    HomeBrewV2 = 0x40,
    H264 = 0x80,
    All = 0xff
}

#[repr(u8)]
#[derive(Debug)]
enum VideoFunction {
    ClientSeek = 0x01
}

#[repr(u8)]
#[derive(Debug)]
enum ObjectEncoding {
    Amf0,
    Amf3 = 0x03
}

#[derive(Debug)]
enum CommandObject {
    App(String),
    FlashVer(String),
    SwfUrl(String),
    TcUrl(String),
    Fpad(bool),
    AudioCodecs(AudioCodecs),
    VideoCodecs(VideoCodecs),
    VideoFunction(VideoFunction),
    PageUrl(String),
    ObjectEncoding(ObjectEncoding)
}

type Arguments = PhantomData<()>;

#[derive(Debug)]
enum Level {
    Status,
    Warning,
    Error
}

#[derive(Debug)]
enum CallResult {
    Failed,
    BadVersion
}

#[derive(Debug)]
enum ConnectResult {
    AppShutdown,
    Closed,
    Failed,
    Rejected,
    Success,
    InvalidApp
}

#[derive(Debug)]
enum NetConnection {
    Call(CallResult),
    Connect(ConnectResult)
}

#[derive(Debug)]
enum ClearResult {
    Success,
    Failed
}

#[derive(Debug)]
enum PublishResult {
    Start,
    BadName
}

#[derive(Debug)]
enum UnpublishResult {
    Success
}

#[derive(Debug)]
enum RecordResult {
    Start,
    NoAccess,
    Stop,
    Failed
}

#[derive(Debug)]
enum BufferResult {
    Empty
}

#[derive(Debug)]
enum PlayResult {
    InsufficientBw,
    Start,
    StreamNotFound,
    Stop,
    Failed,
    Reset,
    PublishNotify,
    UnpublishNotify,
    Switch,
    Transition,
    TransitionComplete,
    Complete,
    FileStructureInvalid,
    NoSupportedTrackFound
}

#[derive(Debug)]
enum TransitionResult {
    Success,
    Forced
}

#[derive(Debug)]
enum SeekResult {
    Notify,
    Failed
}

#[derive(Debug)]
enum PauseResult {
    Notify
}

#[derive(Debug)]
enum UnpauseResult {
    Notify
}

#[derive(Debug)]
enum DataResult {
    Start
}

#[derive(Debug)]
enum NetStream {
    InvalidArg,
    Failed,
    Clear(ClearResult),
    Publish(PublishResult),
    Unpublish(UnpublishResult),
    Record(RecordResult),
    Buffer(BufferResult),
    Play(PlayResult),
    Transition(TransitionResult),
    Seek(SeekResult),
    Pause(PauseResult),
    Unpause(UnpauseResult),
    Data(DataResult)
}

#[derive(Debug)]
enum ScriptResult {
    Error,
    Warning
}

#[derive(Debug)]
enum ResourceResult {
    LowMemory
}

#[derive(Debug)]
enum Application {
    Shutdown,
    Gc,
    Script(ScriptResult),
    Resource(ResourceResult)
}

#[derive(Debug)]
enum SharedObject {
    NoReadAccess,
    NoWriteAccess,
    ObjectCreationFailed,
    BadPersistence
}

#[derive(Debug)]
enum Code {
    NetConnection(NetConnection),
    NetStream(NetStream),
    Application(Application),
    SharedObject(SharedObject),
}

#[derive(Debug)]
struct InfoObject {
    level: Level,
    code: Code,
    description: String
}

#[derive(Debug)]
enum Parameter {
    Len(usize),
    Offset(usize),
    OldStreamName(String),
    Start(i64),
    StreamName(String),
    Transition(String)
}

#[derive(Debug)]
enum PublishingType {
    Record,
    Append,
    Live
}

#[derive(Debug)]
enum InvokeData {
    Connect {
        name: String,
        transaction_id: u32,
        command_object: Vec<CommandObject>,
        optional_arguments: Option<Arguments>
    },
    Call {
        name: String,
        transaction_id: u32,
        command_object: Option<Vec<CommandObject>>,
        optional_arguments: Option<Arguments>
    },
    CreateStream {
        name: String,
        transaction_id: u32,
        command_object: Option<Vec<CommandObject>>
    },
    OnStatus {
        name: String,
        transaction_id: u32,
        command_object: Option<Vec<CommandObject>>,
        info_object: InfoObject
    },
    Play {
        name: String,
        transaction_id: u32,
        command_object: Option<Vec<CommandObject>>,
        stream_name: String,
        start: i64,
        duration: i64,
        reset: bool
    },
    Play2 {
        name: String,
        transaction_id: u32,
        command_object: Option<Vec<CommandObject>>,
        parameters: Vec<Parameter>
    },
    DeleteStream {
        name: String,
        transaction_id: u32,
        command_object: Option<Vec<CommandObject>>
        stream_id: u32
    },
    ReceiveAudio {
        name: String,
        transaction_id: u32,
        command_object: Option<Vec<CommandObject>>,
        received: bool
    },
    ReceiveVideo {
        name: String,
        transaction_id: u32,
        command_object: Option<Vec<CommandObject>>,
        received: bool
    },
    Publish {
        name: String,
        transaction_id: u32,
        publishing_name: String,
        publishing_type: PublishingType,
    }
    Seek {
        name: String,
        transaction_id: u32,
        command_object: Option<Vec<CommandObject>>,
        milli_seconds: Duration
    }
    Pause {
        name: String,
        transaction_id: u32,
        command_object: Option<Vec<CommandObject>>,
        paused: bool,
        milli_seconds: Duration
    }
}

#[derive(Debug)]
enum ChunkData {
    ChunkSize {
        header: RtmpMessageHeader,
        chunk_size: usize
    },
    Abort {
        header: RtmpMessageHeader,
        stream_id: u32
    },
    BytesRead {
        header: RtmpMessageHeader,
        sequence_number: u32
    },
    Ping {
        header: RtmpMessageHeader,
        data: PingData
    },
    ServerBandwidth {
        header: RtmpMessageHeader,
        window_size: usize
    },
    ClientBandwidth {
        header: RtmpMessageHeader,
        window_size: usize,
        limit_type: LimitType
    },
    
    Audio {
        header: RtmpMessageHeader,
        data: Vec<u8>,
        zeroes: Vec<u8>,
        timestamp: Duration,
        size: usize,
        suffix: usize,
        audio_type: u8
    },
    Video {
        header: RtmpMessageHeader,
        data: Vec<u8>,
        zeroes: Vec<u8>,
        timestamp: Duration,
        size: usize,
        suffix: usize,
        video_type: u8,
    },
    FlexSharedObject {
        header: RtmpMessageHeader,
        name: String,
        version: u64,
        persistent: bool,
        events: Vec<SharedObjectEvent>
    },
    FlexMessage {
        header: RtmpMessageHeader,
        flex_byte: usize,
        action: String,
        transaction_id: u32,
        params: Vec<AMF3Object>
    },
    Notify {
        header: RtmpMessageHeader,
        command: NotifyCommand,
        status: String
    },
    SharedObject {
        header: RtmpMessageHeader,
        name: String,
        version: u64,
        perssitent: bool,
        events: Vec<SharedObjectEvent>
    },
    Invoke {
        header: RtmpMessageHeader,
        data: InvokeData
    },
    Aggregate {
        header: RTMPMessageHeader,
        bodies: Vec<AggregateBody>
    }
}

#[derive(Debug)]
struct Chunk {
    basic: BasicHeader,
    message: ChunkMessageHeader,
    data: ChunkData
}
