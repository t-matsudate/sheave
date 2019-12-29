//! # The chunk message patterns
//!
//! After doing the handshake, the server/the client will send actual messages each other.
//! The message consists of following respectively:
//!
//! 1. The chunk basic header
//! 2. The chunk message header
//! 3. The extended timestamp
//! 4. The chunk data
//!
//! ## The chunk basic header
//!
//! This will indicate the chunk stream id and the format of the chunk message header.
//! This will hold following values respectively:
//!
//! 1. The format of the chunk message header (2 bits)
//! 2. The chunk stream id (6 bits, 1 byte or 2 bytes)
//!
//! ### The format of the chunk message header
//!
//! The chunk message header will rely on this field for its length.
//! The correspondence of the number to the chunk message header is following:
//!
//! |Number|Length (in bytes)|
//! | ---: | --------------: |
//! |0     |11               |
//! |1     |7                |
//! |2     |3                |
//! |3     |0                |
//!
//! ### The chunk stream id
//!
//! This will be used to store last chunks every this id.
//! Note following points:
//!
//! * When this is below 64, input following the format bits as 6 bits.
//! * When this is above 64, this will be input in next 1 byte.
//!   * Both (the server and the client) will regard this as what lessened just 64.
//!   * If this is 64 to 319, input **0** as 6 bits following the format bits.  
//!   This is **Big Endian**.
//!   * If this is 319 to 65599, input **1** as as 6 bits following the format bits.  
//!   This is **Little Endian**.
//!
//! ## The chunk message header
//!
//! This will indicate the way to handle the chunk data.
//! This will hold following values respectively:
//!
//! 1. Timestamp (3 bytes)
//! 2. Message length (3 bytes)
//! 3. Message type (1 byte)
//! 4. Chunk message id (4 bytes)
//!
//! ### Timestamp
//!
//! The timestamp when this chunk is sent.
//! Note following points:
//!
//! * If this will exceed 3 bytes, input this to the extended timestamp field instead.
//!   * In this case, input `0xFFFFFF` (just the maximum value of 3 bytes) in this.
//! * This can be 0 in some cases.
//!
//! ### Message length
//!
//! The length of the chunk data.
//! Note following points:
//!
//! * This won't mean that is the total length of the chunk data.
//!   * This won't be considered to count the chunk header of format 3 contained in the chunk data.
//!   * We will be required to remove or to ignore it because it is contained at 1 byte after specified chunk size.
//!   * If we input total size contained the chunk header of format 3, many of products will output the error probably. 
//!
//! ### Message type
//!
//! The type of the chunk data.
//! The correspondence of the number to the chunk data is following:
//!
//! |Number|Message Type        |Length (in byte)|
//! | ---: | :----------------- | -------------: |
//! |1     |Chunk size          |4               |
//! |2     |Abort               |4               |
//! |3     |Bytes read          |4               |
//! |4     |Ping                |Variable        |
//! |5     |Server bandwidth    |4               |
//! |6     |Client bandwidth    |5               |
//! |8     |Audio               |Variable        |
//! |9     |Video               |Variable        |
//! |15    |Notify (AMF3)       |Variable        |
//! |16    |Shared object (AMF3)|Variable        |
//! |17    |Invoke (AMF3)       |Variable        |
//! |18    |Notify (AMF0)       |Variable        |
//! |19    |Shared object (AMF0)|Variable        |
//! |20    |Invoke (AMF0)       |Variable        |
//! |22    |Metadata            |Variable        |
//!
//! ### Message stream id
//!
//! The id to identify the user who sent this chunk.
//! Note following points:
//!
//! * This can be 0 in some cases.
//! * This must be emitted by the server when received `Invoke(createStream)`.
//! * This is **Little Endian**.
//!
//! The patterns of the chunk message header every the format are following:
//!
//! |Field            |Format 0|Format 1|Format 2|Format 3|
//! | :-------------- | :----: | :----: | :----: | :----: |
//! |Timestamp        |✔       |✔       |✔       |-       |
//! |Message length   |✔       |✔       |-       |-       |
//! |Message type     |✔       |✔       |-       |-       |
//! |Message stream id|✔       |-       |-       |-       |
//!
//! ## The extended timestamp (4 bytes)
//!
//! When the timestamp has exceeded 3 bytes, input it in this field instead.
//! Note that mustn't input if it hasn't exceeded 3 bytes. If input in its case, the program can misunderstand as the chunk data.
//!
//! ## The chunk data
//!
//! The content of this chunk.
//! The patterns are following:
//!
//! ### Chunk size
//!
//! This will indicate the size what will split the chunk data.
//! The server/the client will be required to insert the chunk header of format 3 per who specified chunk size.
//! In the official specification paper, this is specified that the most significant bit must be 0. (However we won't be required to care this normally because the message length will be represented at most 3 bytes.)
//!
//! ### Abort
//!
//! This will be input the chunk stream id which will abort to send/receive.
//! When we received this chunk data, we are required to stop to send/receive the chunk data related to this chunk stream id.
//! However in the FFmpeg and in the Open Broadcaster Software, both have never used this chunk data yet.
//!
//! ### Bytes read
//!
//! This will be input the total message length which has receive until now.
//! The server/the client must send this chunk data each other whenever read byte size reached specified server bandwidth/client baandwidth.
//!
//! ### Ping
//!
//! This will be used to confirm current state for the server/the client each other.
//! The values to be required to input are following respectively:
//!
//! 1. Event type (2 bytes)
//!
//! This will indicate the kinds of data following this bytes.
//! The correspondence of the number to actual event data is following:
//!
//! |Number|Event type               |Length (in byte)|
//! | ---: | :---------------------- | -------------: |
//! |0     |Stream begin             |4               |
//! |1     |Stream EOF               |4               |
//! |2     |Stream dry               |4               |
//! |3     |Buffer length            |8               |
//! |4     |Stream is recorded       |4               |
//! |6     |Ping                     |4               |
//! |7     |Pong                     |4               |
//! |26    |SWF verification request |0               |
//! |27    |SWF verification response|42              |
//! |31    |Buffer empty             |4               |
//! |32    |Buffer ready             |4               |
//!
//! 2. Event data
//!
//! This will be input actual ping data.
//!
//! * Stream begin, Stream EOF, Stream dry, Stream is recorded, Buffer empty and Buffer ready.
//!
//! This will be input only the chunk message id (4 bytes).
//! In the phase of the application connection, the chunk message id will be 0 inevitably.
//!
//! * Buffer length
//!
//! This will be input the chunk message id (4 bytes) and the *buffer* length (4 bytes).
//! Note that this has differed from the chunk size.
//! This has indicated the size to send/to receive the data in millisecond.
//!
//! * Ping and Pong
//!
//! This will be input the timestamp when the server emitted this event.
//!
//! * SWF verification request
//!
//! This will be input no data.
//!
//! * SWF verification response
//!
//! This will be input the HMAC-SHA256 signature what generated from the SWF data.
//! Note that we will be required to input following bytes respectively before its signature:
//!
//! 1. 1 (1 byte)
//! 2. 1 (1 byte)
//! 3. the size of raw SWF data. (4 bytes)
//! 4. the size of raw SWF data. (4 bytes)
//!
//! ### Server bandwidth
//!
//! This will be input the limit of the server side bandwidth.
//!
//! ### Client bandwidth
//!
//! This will be input the limit of the client side bandwidth and the limit type.
//! The limit type is following:
//!
//! * Hard
//! * Soft
//! * Dynamic
//!
//! #### Hard
//!
//! This indicates that the receiver should limit the bandwidth to this.
//!
//! #### Soft
//!
//! This indicates that the receiver should limit the bandwidth to either this value or already received one, whichever is smaller.
//!
//! #### Dynamic
//!
//! This indicates that the receiver should regard this as the limit type of hard if the previous limit type was hard, otherwise ignore this message.
//!
//! The correspondence of the number to the limit type is following:
//!
//! |Number|Limit type|
//! | ---: | :------- |
//! |0     |Hard      |
//! |1     |Soft      |
//! |2     |Dynamic   |
//!
//! ### Audio
//!
//! This will be input the audio data.
//!
//! ### Video
//!
//! This will be input the video data.
//!
//! Note: the format for audio/video has differed every codec.
//!
//! ### Notify
//!
//! Currently, this will be input the metadata of audio/video data.
//!
//! ### Shared Object
//!
//! This will be input the information to share with the client or other server instances.
//! However the client side applications published as the OSS haven't implemented this chunk data yet.
//!
//! ### Invoke
//!
//! This will be input the information to need what the server/the client application each will succeed to connect.
//!
//! ### Metadata
//!
//! This will be input the data to aggregate the audio, the video, or the notify(metadata) chunks.
use std::{
    collections::{
        HashMap
    },
    time::{
        Duration
    }
};

/// # The maximum length of the message length
pub(crate) const U24_MAX: u32 = 0x00ffffff;

/// # The format of the chunk message header
///
/// The correspondence of the number to this enum is following:
///
/// |Number|MessageFormat |
/// | ---: | :----------- |
/// |0     |`New`         |
/// |1     |`SameSource`  |
/// |2     |`TimerChange` |
/// |3     |`Continue`    |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>`, and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MessageFormat {
    New,
    SameSource,
    TimerChange,
    Continue
}

impl From<u8> for MessageFormat {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `message_format: u8`
    ///
    /// The number to indicate the message header's format.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value above 3.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::MessageFormat;
    ///
    /// let new: MessageFormat = (0 as u8).into();
    /// let same_source: MessageFormat = (1 as u8).into();
    /// let timer_change: MessageFormat = (2 as u8).into();
    /// let message_continue: MessageFormat = (3 as u8).into();
    ///
    /// /* This will print `New`. */
    /// println!("{:?}", new);
    /// /* This will print `SameSource`. */
    /// println!("{:?}", same_source);
    /// /* This will print `TimerChange`. */
    /// println!("{:?}", timer_change);
    /// /* This will print `Continue`. */
    /// println!("{:?}", message_continue);
    /// ```
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

/// # The chunk stream id
///
/// If the chunk stream id is 0 to 255, this will be used as `ChunkId::U8(n)`.
/// If it is above 255, this will be used as `ChunkId::U16(n)`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ChunkId {
    U8(u8),
    U16(u16)
}

impl Default for ChunkId {
    /// Constructs a new `ChunkID` with its default value, for constructing a new `RtmpHandler`.
    fn default() -> Self {
        ChunkId::U8(u8::default())
    }
}

/// # The chunk basic header
///
/// This consists of following data:
///
/// |Field          |Type           |
/// | :------------ | :------------ |
/// |message\_format|`MessageFormat`|
/// |chunk\_id      |`ChunkId`      |
///
/// [`MessageFormat`]: ./enum.MessageFormat.html
/// [`ChunkId`]: ./enum.ChunkId.html
#[derive(Debug, Clone, Copy)]
pub struct BasicHeader {
    message_format: MessageFormat,
    chunk_id: ChunkId
}

impl BasicHeader {
    /// The bits to indicate the format of the chunk message header.
    pub(crate) const MESSAGE_HEADER_FORMAT: u8 = 0xc0;
    /// The bits to indicate the chunk id.
    pub(crate) const BASIC_HEADER_TYPE: u8 = 0x3f;
    /// The length when following patterns:
    ///
    /// * The first 1 byte of the chunk basic header.
    /// * The remaining length of the chunk basic header when the chunk stream id is 64 to 319.
    pub(crate) const LEN_ONE_BYTE: usize = 1;
    /// The remaining length of the chunk basic header when the chunk stream id is 320 to 65599.
    pub(crate) const LEN_TWO_BYTES: usize = 2;

    /// Constructs a new `BasicHeader`.
    ///
    /// # Parameters
    ///
    /// * `message_format: MessageFormat`
    ///
    /// The enum of the format of the chunk message header.
    ///
    /// * `chunk_id: ChunkId`
    ///
    /// The enum of the chunk stream id.
    pub fn new(message_format: MessageFormat, chunk_id: ChunkId) -> Self {
        BasicHeader { message_format, chunk_id }
    }

    /// Returns the format of the chunk message header.
    pub fn get_message_format(&self) -> MessageFormat {
        self.message_format
    }

    /// Returns the chunk id.
    pub fn get_chunk_id(&self) -> ChunkId {
        self.chunk_id
    }
}

/// # The message types of the chunk message header
///
/// The correspondence of the number to this enum is following:
///
/// |Number|MessageType      |
/// | ---: | :-------------- |
/// |1     |`ChunkSize`      |
/// |3     |`BytesRead`      |
/// |4     |`Ping`           |
/// |5     |`ServerBandwidth`|
/// |6     |`ClientBandwidth`|
/// |8     |`Audio`          |
/// |9     |`Video`          |
/// |18    |`Notify` (AMF0)  |
/// |20    |`Invoke` (AMF0)  |
/// |Other |`Unknown`        |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>`, and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    ChunkSize = 0x01,
    BytesRead = 0x03,
    Ping,
    ServerBandwidth,
    ClientBandwidth,
    Audio = 0x08,
    Video,
    Notify = 0x12,
    Invoke = 0x14,
    Unknown
}

impl From<u8> for MessageType {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `message_type_id: u8`
    ///
    /// The number to indicate the message type.
    ///
    /// Note that this will regard the message type as the `Unknown` if the value of neither 1, 3, 4, 5, 6, 8, 9, 18, nor 20 is passed.
    /// the Abort (2), the Shared Object (19), the MetaData/the Aggregate (22) and the AMF3's messages have implemented yet.
    /// Due to following:
    ///
    /// * the Abort (2), the SharedObject (19), and the AMF3's messages (i.e. 15, 16 and 17) haven't been used in the client side yet.
    /// * the MetaData/the Aggregate (22) has been implemented in the client side but has been sent from it yet.
    ///
    /// 0, 7, 10, 11, 12, 13, 14, and 21 are undefined in the first place.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::MessageType;
    ///
    /// let chunk_size: MessageType = (1 as u8).into();
    /// let bytes_read: MessageType = (3 as u8).into();
    /// let ping: MessageType = (4 as u8).into();
    /// let server_bandwidth: MessageType = (5 as u8).into();
    /// let client_bandwidth: MessageType = (6 as u8).into();
    /// let audio: MessageType = (8 as u8).into();
    /// let video: MessageType = (9 as u8).into();
    /// let notify: MessageType = (18 as u8).into();
    /// let invoke: MessageType = (20 as u8).into();
    /// let unknown: MessageType = (0 as u8).into();
    ///
    /// /* This will print `ChunkSize`. */
    /// println!("{:?}", chunk_size);
    /// /* This will print `BytesRead`. */
    /// println!("{:?}", bytes_read);
    /// /* This will print `Ping`. */
    /// println!("{:?}", ping);
    /// /* This will print `ServerBandwidth`. */
    /// println!("{:?}", server_bandwidth);
    /// /* This will print `ClientBandwidth`. */
    /// println!("{:?}", client_bandwidth);
    /// /* This will print `Audio`. */
    /// println!("{:?}", audio);
    /// /* This will print `Video`. */
    /// println!("{:?}", video);
    /// /* This will print `Notify`. */
    /// println!("{:?}", notify);
    /// /* This will print `Invoke`. */
    /// println!("{:?}", invoke);
    /// /* This will print `Unknown`. */
    /// println!("{:?}", unknown);
    /// ```
    fn from(message_type_id: u8) -> MessageType {
        use MessageType::*;

        match message_type_id {
            0x01 => ChunkSize,
            0x03 => BytesRead,
            0x04 => Ping,
            0x05 => ServerBandwidth,
            0x06 => ClientBandwidth,
            0x08 => Audio,
            0x09 => Video,
            0x12 => Notify,
            0x14 => Invoke,
            _ => Unknown
        }
    }
}

/// # The chunk message header
///
/// This consists of following patterns:
///
/// * The format 0 (New)
/// * The format 1 (SameSource)
/// * The format 2 (TimerChange)
/// * The format 3 (Continue)
///
/// ## The format 0 (New)
///
/// The format 0 consists of following data:
///
/// 1. Timestamp
/// 2. Message length
/// 3. Message type
/// 4. Chunk message id
///
/// ## The format 1 (SameSource)
///
/// The format 1 consists of following data:
///
/// 1. Timestamp
/// 2. Message length
/// 3. Message type
///
/// ## The format 2 (TimerChange)
///
/// The format 2 consists of following data:
///
/// 1. Timestamp
///
/// ## The format 3 (Continue)
///
/// The format 3 has no data.
#[derive(Debug, Clone, Copy)]
pub enum MessageHeader {
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
    /// The length of the format 0.
    pub(crate) const LEN_NEW: usize = 11;
    /// The length of the format 1.
    pub(crate) const LEN_SAME_SOURCE: usize = 7;
    /// The length of the format 2.
    pub(crate) const LEN_TIMER_CHANGE: usize = 3;

    /// Returns the message type of the chunk data.
    /// This can return the `None`.
    /// Its cases are following:
    ///
    /// |Pattern      |Return             |
    /// | :---------- | :---------------- |
    /// |`New`        |`Some(MessageType)`|
    /// |`SameSource` |`Some(MessageType)`|
    /// |`TimerChange`|`None`             |
    /// |`Continue`   |`None`             |
    pub fn get_message_type(&self) -> Option<MessageType> {
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

    /// Returns the message length of the chunk data.
    /// This can return the `None`.
    /// Its cases are following:
    ///
    /// |Pattern      |Return     |
    /// | :---------- | :-------- |
    /// |`New`        |`Some(u32)`|
    /// |`SameSource` |`Some(u32)`|
    /// |`TimerChange`|`None`     |
    /// |`Continue`   |`None`     |
    pub fn get_message_len(&self) -> Option<u32> {
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

    /// Returns the timestamp.
    /// This can return the `None`.
    /// Its cases are following:
    ///
    /// |Pattern      |Return          |
    /// | :---------- | :------------- |
    /// |`New`        |`Some(Duration)`|
    /// |`SameSource` |`Some(Duration)`|
    /// |`TimerChange`|`Some(Duration)`|
    /// |`Continue`   |`None`          |
    pub fn get_timestamp(&self) -> Option<Duration> {
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

    /// Returns the chunk message id.
    /// This can return the `None`.
    /// Its cases are following:
    ///
    /// |Pattern      |Return     |
    /// | :---------- | :-------- |
    /// |`New`        |`Some(u32)`|
    /// |`SameSource` |`None`     |
    /// |`TimerChange`|`None`     |
    /// |`Continue`   |`None`     |
    pub fn get_message_id(&self) -> Option<u32> {
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

/// # The AMF types
///
/// This correspondence of the number to this enum is following:
///
/// |Number|AmfDataType  |
/// | ---: | :---------- |
/// |0     |`Number`     |
/// |1     |`Boolean`    |
/// |2     |`String`     |
/// |3     |`Object`     |
/// |5     |`Null`       |
/// |8     |`MixedArray` |
/// |9     |`ObjectEnd`  |
///
/// This enum and the `u8` value can convert into each other because this has implemented `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug)]
pub enum AmfDataType {
    Number,
    Boolean,
    String,
    Object,
    Null,
    MixedArray,
    ObjectEnd
}

impl From<u8> for AmfDataType {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `amf_data_type_id: u8`
    ///
    /// The number to indicate the AMF0's type.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed either 4, 6, 7 or the value above 9.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::AmfDataType;
    ///
    /// let number: AmfDataType = (0 as u8).into();
    /// let boolean: AmfDataType = (1 as u8).into();
    /// let string: AmfDataType = (2 as u8).into();
    /// let object: AmfDataType = (3 as u8).into();
    /// let null: AmfDataType = (5 as u8).into();
    /// let mixed_array: AmfDataType = (8 as u8).into();
    /// let object_end: AmfDataType = (9 as u8).into();
    ///
    /// /* This will print `Number`. */
    /// println!("{:?}", number);
    /// /* This will print `Boolean`. */
    /// println!("{:?}", boolean);
    /// /* This will print `String`. */
    /// println!("{:?}", string);
    /// /* This will print `Object`. */
    /// println!("{:?}", object);
    /// /* This will print `Null`. */
    /// println!("{:?}", null);
    /// /* This will print `MixedArray`. */
    /// println!("{:?}", mixed_array);
    /// /* This will print `ObjectEnd`. */
    /// println!("{:?}", object_end);
    /// ```
    fn from(amf_data_type_id: u8) -> AmfDataType {
        use AmfDataType::*;
        use AmfDataType::String as AmfString;

        match amf_data_type_id {
            0x00 => Number,
            0x01 => Boolean,
            0x02 => AmfString,
            0x03 => Object,
            0x05 => Null,
            0x08 => MixedArray,
            0x09 => ObjectEnd,
            _ => panic!("Undefined amf data type number!")
        }
    }
}

/// # The AMF data
///
/// This consists of following data:
///
/// * `Number`
/// * `Boolean`
/// * `String`
/// * `Object`
/// * `Null`
/// * `MixedArray`
/// * `ObjectEnd`
///
/// ## Number
///
/// 8 bytes floating point number. (i.e. IEEE 754 numbers)
///
/// ## Boolean
///
/// 1 byte number to indicate either `true` or `false`.
/// Usually, if the value is 0 then it's regarded it as `false`, and is regarded it as `true` otherwise.
///
/// ## String
///
/// 2 bytes length string.
/// This consists of following structure respectively:
///
/// 1. Length (2 bytes)
/// 2. Actual string (remaining)
///
/// ## Object
///
/// The name/value pairs not to have their lengths.
/// The name is 2 bytes string not to have the AMF0's type marker.
/// The value is some `AmfData`.
///
/// ## Null
///
/// This has no value.
/// This indicates that its field has been input no value.
/// e.g. the `CommandObject` after the connect invoking
///
/// ## MixedArray
///
/// The name/value pairs to have their lengths. (like the ECMA's array)
/// This is the same as the `Object` type except this has 2 byte length at its head.
///
/// ## ObjectEnd
///
/// This has no value but will be used at the end of following AMF0 types:
///
/// * `Object`
/// * `MixedArray`
/// * `NamedObject`
///
/// Above AMF0 types need the marker to indicate its end because they don't have the length.
/// The marker consists of following structure respectively:
///
/// 1. The AMF0's String type (2 bytes, but empty)
/// 2. `0x09` (just the id for this type)
#[derive(Debug, Clone)]
pub enum AmfData {
    Number(f64),
    Boolean(bool),
    String(String),
    Object(HashMap<String, AmfData>),
    Null,
    MixedArray(HashMap<String, AmfData>),
    ObjectEnd,
}

impl AmfData {
    /// Actual object end marker.
    /// i.e. `[0x00, 0x00, 0x09]` (following after the empty string)
    pub(crate) const OBJECT_END_SEQUENCE: [u8; 3] = [0, 0, AmfDataType::ObjectEnd as u8];

    /// Returns the number value if this is the AMF's `Number` type.
    /// Otherwise returns the `None`.
    pub fn number(self) -> Option<f64> {
        match self {
            AmfData::Number(n) => Some(n),
            _ => None
        }
    }

    /// Returns the boolean value if this is the AMF's `Boolean` type.
    /// Otherwise returns the `None`.
    pub fn boolean(self) -> Option<bool> {
        match self {
            AmfData::Boolean(b) => Some(b),
            _ => None
        }
    }

    /// Returns the string value if this is the AMF's `String` type.
    /// Otherwise returns the `None`.
    pub fn string(self) -> Option<String> {
        match self {
            AmfData::String(s) => Some(s),
            _ => None
        }
    }

    /// Returns the object value if this is the AMF's `Object` type.
    /// Otherwise returns the `None`.
    pub fn object(self) -> Option<HashMap<String, AmfData>> {
        match self {
            AmfData::Object(o) | AmfData::MixedArray(o) => Some(o),
            _ => None
        }
    }
}

/// This is the optional argumeent in the connect of the NetConnection command.
/// This hasn't been checked its exsitence yet.
#[derive(Debug, Clone)]
pub struct Argument;

/// # The audio codec patterns
///
/// This indicates the audio codec to input in the command object.
/// The correspondence of the number to this enum is following:
///
/// |Number|AudioCodec|
/// | ---: | :------- |
/// |0x0001|`None`    |
/// |0x0002|`Adpcm`   |
/// |0x0004|`Mp3`     |
/// |0x0008|`Intel`   |
/// |0x0010|`Unused`  |
/// |0x0020|`Nerry8`  |
/// |0x0040|`Nerry`   |
/// |0x0080|`G711a`   |
/// |0x0100|`G711u`   |
/// |0x0200|`Nerry16` |
/// |0x0400|`Aac`     |
/// |0x0800|`Speex`   |
/// |0x0fff|`All`     |
///
/// This enum and the `u16` value can convert into each other because this has implemented the `From<u16>` and has set the `#[repr(u16)]` attribute.
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum AudioCodec {
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
    /// Converts the `u16` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `codec_flag: u16`
    ///
    /// The number to indicate the audio codec.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value above 0x0fff or is turned plural bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::AudioCodec;
    ///
    /// /* Note that the audio codecs are 16 bits flag. */
    /// let none: AudioCodec = (0x0001 as u16).into();
    /// let adpcm: AudioCodec = (0x0002 as u16).into();
    /// let mp3: AudioCodec = (0x0004 as u16).into();
    /// let intel: AudioCodec = (0x0008 as u16).into();
    /// let unused: AudioCodec = (0x0010 as u16).into();
    /// let nerry8: AudioCodec = (0x0020 as u16).into();
    /// let nerry: AudioCodec = (0x0040 as u16).into();
    /// let g711a: AudioCodec = (0x0080 as u16).into();
    /// let g711u: AudioCodec = (0x0100 as u16).into();
    /// let nerry16: AudioCodec = (0x0200 as u16).into();
    /// let aac: AudioCodec = (0x0400 as u16).into();
    /// let speex: AudioCodec = (0x0800 as u16).into();
    /// let all: AudioCodec = (0x0fff as u16).into();
    ///
    /// /* This will print `None`. */
    /// println!("{:?}", none);
    /// /* This will print `Adpcm`. */
    /// println!("{:?}", adpcm);
    /// /* This will print `Mp3`. */
    /// println!("{:?}", mp3);
    /// /* This will print `Intel`. */
    /// println!("{:?}", intel);
    /// /* This will print `Unused`. */
    /// println!("{:?}", unused);
    /// /* This will print `Nerry8`. */
    /// println!("{:?}", nerry8);
    /// /* This will print `Nerry`. */
    /// println!("{:?}", nerry);
    /// /* This will print `G711a`. */
    /// println!("{:?}", g711a);
    /// /* This will print `G711u`. */
    /// println!("{:?}", g711u);
    /// /* This will print `Nerry16`. */
    /// println!("{:?}", nerry16);
    /// /* This will print `Aac`. */
    /// println!("{:?}", aac);
    /// /* This will print `Speex`. */
    /// println!("{:?}", speex);
    /// /* This will print `All`. */
    /// println!("{:?}", all);
    /// ```
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

/// Converts the 16 bits flags for the audio codecs into the `Vec<AudioCodec>`.
///
/// # Parameters
///
/// * `codec_flags: u16`
///
/// The flags to indicate the supported audio codecs.
/// See the `AudioCodec` for more detail about each audio codec flag.
///
/// # Panics
///
/// This will emit the `panic!` if is passed the value above 0x0fff.
/// Because it has exceeded the range for the audio codec flag.
///
/// # Examples
///
/// ```
/// use sheave::messages::{
///     AudioCodec,
///     detect_audio_codecs
/// };
///
/// /* These flags indicate all but `Intel` and `Unused`. */
/// let audio_codecs = detect_audio_codecs(0x0fe7);
///
/// /*
///  * This will print following format:
///  * [
///  *     None,
///  *     Adpcm,
///  *     Mp3,
///  *     Nerry8,
///  *     Nerry,
///  *     G711a,
///  *     G711u,
///  *     Nerry16,
///  *     Aac,
///  *     Speex
///  * ]
/// */
/// println!("{:?}", audio_codecs);
/// ```
///
/// [`AudioCodec`]: ./enum.AudioCodec.html
pub fn detect_audio_codecs(codec_flags: u16) -> Vec<AudioCodec> {
    if codec_flags > 0x0fff {
        panic!("the audio codec flag's value is above 0x0fff!");
    }

    let mut flag: u16 = 1;
    let mut v: Vec<AudioCodec> = Vec::new();

    // The flag of `AudioCodec::All` can't be detected in 1 bit.
    if codec_flags == 0x0fff {
        v.push(AudioCodec::All);
        return v;
    }

    while flag <= 0x0800 {
        let detected = codec_flags & flag;

        if detected != 0 {
            v.push(detected.into());
        }

        flag <<= 1;
    }

    v
}

/// Converts the audio codecs into the 16 bits flags.
///
/// # Parameters
///
/// * `audio_codecs: Vec<AudioCodec>`
///
/// The supported audio codecs.
/// See the `AudioCodec` for more detail about each audio codec flag.
///
/// # Examples
///
/// ```
/// use sheave::messages::{
///     AudioCodec,
///     convert_audio_codecs_into_flags
/// };
///
/// let mut audio_codecs: Vec<AudioCodec> = Vec::new();
///
/// audio_codecs.push(AudioCodec::None);
/// audio_codecs.push(AudioCodec::Adpcm);
/// audio_codecs.push(AudioCodec::Mp3);
/// audio_codecs.push(AudioCodec::Nerry8);
/// audio_codecs.push(AudioCodec::Nerry);
/// audio_codecs.push(AudioCodec::G711a);
/// audio_codecs.push(AudioCodec::G711u);
/// audio_codecs.push(AudioCodec::Nerry16);
/// audio_codecs.push(AudioCodec::Aac);
/// audio_codecs.push(AudioCodec::Speex);
///
/// let flags = convert_audio_codecs_into_flags(audio_codecs);
///
/// /* This will print `0x0fe7`. */
/// println!("{}", flags);
/// ```
///
/// [`AudioCodec`]: ./enum.AudioCodec.html
pub fn convert_audio_codecs_into_flags(audio_codecs: Vec<AudioCodec>) -> u16 {
    let mut flags: u16 = 0;

    for audio_codec in audio_codecs {
        flags |= audio_codec as u16;
    }

    flags
}

/// # The video codec patterns
///
/// The correspondence of the number to this enum is following:
///
/// |Number|VideoCodec  |
/// | ---: | :--------- |
/// |0x01  |`Unused`    |
/// |0x02  |`Jpeg`      |
/// |0x04  |`Sorenson`  |
/// |0x08  |`Homebrew`  |
/// |0x10  |`Vp6`       |
/// |0x20  |`Vp6a`      |
/// |0x40  |`Homebrewv2`|
/// |0x80  |`H264`      |
/// |0xff  |`All`       |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VideoCodec {
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
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `codec_flag: u8`
    ///
    /// The number to indicate the supported video codecs.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is turned plural bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::VideoCodec;
    ///
    /// let unused: VideoCodec = (0x01 as u8).into();
    /// let jpeg: VideoCodec = (0x02 as u8).into();
    /// let sorenson: VideoCodec = (0x04 as u8).into();
    /// let homebrew: VideoCodec = (0x08 as u8).into();
    /// let vp6: VideoCodec = (0x10 as u8).into();
    /// let vp6a: VideoCodec = (0x20 as u8).into();
    /// let homebrew_v2: VideoCodec = (0x40 as u8).into();
    /// let h264: VideoCodec = (0x80 as u8).into();
    /// let all: VideoCodec = (0xff as u8).into();
    ///
    /// /* This will print `Unused`. */
    /// println!("{:?}", unused);
    /// /* This will print `Jpeg`. */
    /// println!("{:?}", jpeg);
    /// /* This will print `Sorenson`. */
    /// println!("{:?}", sorenson);
    /// /* This will print `Homebrew`. */
    /// println!("{:?}", homebrew);
    /// /* This will print `Vp6`. */
    /// println!("{:?}", vp6);
    /// /* This will print `Vp6a`. */
    /// println!("{:?}", vp6a);
    /// /* This will print `Homebrewv2`. */
    /// println!("{:?}", homebrew_v2);
    /// /* This will print `H264`. */
    /// println!("{:?}", h264);
    /// /* This will print `All`. */
    /// println!("{:?}", all);
    /// ```
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

/// Converts the 8 bits flags for the video codecs into the `Vec<VideoCodec>`.
///
/// # Parameters
///
/// * `codec_flags: u8`
///
/// The flags to indicate the supported video codecs.
/// See the `VideoCodec` for more detail about each video codec flag.
///
/// # Examples
///
/// ```
/// use sheave::messages::{
///     VideoCodec,
///     detect_video_codecs
/// };
///
/// /* These flags indicate all but `Unused` and `Jpeg`. */
/// let video_codecs = detect_video_codecs(0xfc);
///
/// /*
///  * This will print following format:
///  * [
///  *     Sorenson,
///  *     Homebrew,
///  *     Vp6,
///  *     Vp6a,
///  *     Homebrewv2,
///  *     H264
///  * ]
/// */
/// println!("{:?}", video_codecs);
/// ```
///
/// [`VideoCodec`]: ./enum.VideoCodec.html
pub fn detect_video_codecs(codec_flags: u8) -> Vec<VideoCodec> {
    let mut flag: u8 = 1;
    let mut v: Vec<VideoCodec> = Vec::new();

    // The flag of `VideoCodec::All` can't also be detected in 1 bit.
    if codec_flags == 0xff {
        v.push(VideoCodec::All);
        return v;
    }

    // The `u8` value can't shift larger than 0x80.
    while flag < 0x80 {
        let detected = codec_flags & flag;

        if detected != 0 {
            v.push((codec_flags & flag).into());
        }

        flag <<= 1;
    }

    // Repeats the same one for the most significant bit.
    let detected = codec_flags & flag;

    if detected != 0 {
        v.push(detected.into());
    }

    v
}

/// Converts the video codecs into the 8 bits flags.
///
/// # Parameters
///
/// * `video_codecs: u8`
///
/// The supported video codecs.
/// See the `VideoCodec` for more detail about each video codec flag.
///
/// # Examples
///
/// ```
/// use::sheave::messages::{
///     VideoCodec,
///     convert_video_codecs_into_flags
/// };
///
/// let mut video_codecs: Vec<VideoCodec> = Vec::new();
///
/// video_codecs.push(VideoCodec::Sorenson);
/// video_codecs.push(VideoCodec::Homebrew);
/// video_codecs.push(VideoCodec::Vp6);
/// video_codecs.push(VideoCodec::Vp6a);
/// video_codecs.push(VideoCodec::Homebrewv2);
/// video_codecs.push(VideoCodec::H264);
///
/// let flags = convert_video_codecs_into_flags(video_codecs);
///
/// /* This will print `0xfc`. */
/// println!("{}", flags);
/// ```
///
/// [`VideoCodec`]: ./enum.VideoCodec.html
pub fn convert_video_codecs_into_flags(video_codecs: Vec<VideoCodec>) -> u8 {
    let mut flags: u8 = 0;

    for video_codec in video_codecs {
        flags |= video_codec as u8;
    }

    flags

}

/// # The video function pattern
///
/// The correspondence of the number to this enum is following:
///
/// |Number|VideoFunction|
/// | ---: | :---------- |
/// |0x01  |`ClientSeek` |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VideoFunction {
    ClientSeek = 1
}

impl From<u8> for VideoFunction {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `video_function: u8`
    ///
    /// The number to indicate that has been supported the video function.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed either 0 or the value above 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::VideoFunction;
    ///
    /// let client_seek: VideoFunction = (1 as u8).into();
    ///
    /// /* This will print `ClientSeek`. */
    /// println!("{:?}", client_seek);
    /// ```
    fn from(video_function: u8) -> VideoFunction {
        use VideoFunction::*;

        match video_function {
            1 => ClientSeek,
            _ => panic!("Undefined video function flag!")
        }
    }
}

/// # The object encoding pattern
///
/// The correspondence of the number to this enum is following:
///
/// |Number|ObjectEncoding|
/// | ---: | :----------- |
/// |0x00  |`Amf0`        |
/// |0x03  |`Amf3`        |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ObjectEncoding {
    Amf0,
    Amf3 = 3
}

impl From<u8> for ObjectEncoding {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `encoding: u8`
    ///
    /// The number to indicate the Action Message Format's version.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value neither 0 nor 3.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::ObjectEncoding;
    ///
    /// let amf0: ObjectEncoding = (0 as u8).into();
    /// let amf3: ObjectEncoding = (3 as u8).into();
    ///
    /// /* This will print `Amf0`. */
    /// println!("{:?}", amf0);
    /// /* This will print `Amf3`. */
    /// println!("{:?}", amf3);
    /// ```
    fn from(encoding: u8) -> ObjectEncoding {
        use ObjectEncoding::*;

        match encoding {
            0 => Amf0,
            3 => Amf3,
            _ => panic!("Undefined encoding type!")
        }
    }
}

/// # The command object
///
/// This consists of following data:
///
/// |Field           |Type                    |
/// | :------------- | :--------------------- |
/// |fpad            |`Option<bool>`          |
/// |object\_encoding|`Option<ObjectEncoding>`|
/// |video\_function |`Option<VideoFunction>` |
/// |video\_codec    |`Option<VideoCodec>`    |
/// |audio\_codec    |`Option<AudioCodec>`    |
/// |app             |`Option<String>`        |
/// |command\_type   |`Option<String>`        |
/// |flash\_ver      |`Option<String>`        |
/// |swf\_url        |`Option<String>`        |
/// |tc\_url         |`Option<String>`        |
/// |page\_url       |`Option<String>`        |
///
/// These fields won't be contained all necessarily.
/// Actually, the command object to be sent from the FFmpeg will be contained just following fields:
///
/// * app
/// * command\_type
/// * flash\_ver
/// * tc\_url
///
/// Because of actual command object is so, all field has been contained in the `Option` type.
/// This type and the `HashMap<String, AmfData>` type can convert into each other because this has implemented the `From<HashMap<String, AmfData>>` and the `From<CommandObject>`.
#[derive(Debug, Clone, Default)]
pub struct CommandObject {
    fpad: Option<bool>,
    object_encoding: Option<ObjectEncoding>,
    video_function: Option<VideoFunction>,
    video_codecs: Option<Vec<VideoCodec>>,
    audio_codecs: Option<Vec<AudioCodec>>,
    app: Option<String>,
    command_type: Option<String>,
    flash_ver: Option<String>,
    swf_url: Option<String>,
    tc_url: Option<String>,
    page_url: Option<String>
}

impl CommandObject {
    /// Constructs a new `CommandObject`.
    pub fn new() -> Self {
        CommandObject {
            fpad: None,
            object_encoding: None,
            video_function: None,
            video_codecs: None,
            audio_codecs: None,
            app: None,
            command_type: None,
            flash_ver: None,
            swf_url: None,
            tc_url: None,
            page_url: None
        }
    }

    /// Sets the fpad.
    ///
    /// # Parameters
    ///
    /// * `fpad: Option<bool>`
    ///
    /// A `bool` value converted from AMF's `Boolean`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Boolean`]: ./enum.AmfData.html#variant.Boolean
    pub fn set_fpad(&mut self, fpad: Option<bool>) {
        self.fpad = fpad;
    }

    /// Sets the object encoding.
    ///
    /// # Parameters
    ///
    /// * `object_encoding: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// This indicates the AMF version which its user uses.
    /// If you set no value, pass the `None`.
    /// See the `ObjectEncoding` for more detail about the object encoding.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    /// [`ObjectEncoding`]: ./enum.ObjectEncoding.html
    pub fn set_object_encoding(&mut self, object_encoding: Option<f64>) {
        self.object_encoding = object_encoding.map(
            |object_encoding| ((object_encoding as u64) as u8).into()
        );
    }

    /// Sets the video function.
    ///
    /// # Parameters
    ///
    /// * `video_function: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// This indicates the special video function in FLV.
    /// If you set no value, pass the `None`.
    /// See the `VideoFunction` for more detail about the video function.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    /// [`VideoFunction`]: ./enum.VideoFunction.html
    pub fn set_video_function(&mut self, video_function: Option<f64>) {
        self.video_function = video_function.map(
            |video_function| ((video_function as u64) as u8).into()
        );
    }

    /// Sets the video codecs.
    ///
    /// # Parameters
    ///
    /// * `video_codecs: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// This indicates supported video codecs.
    /// This can specify plural codecs because will be used as a 64 bits flag.
    /// If you set no value, pass the `None`.
    /// See the `VideoCodec` for more detail about the video codecs flag.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    /// [`VideoCodec`]: ./enum.VideoCodec.html
    pub fn set_video_codecs(&mut self, video_codecs: Option<f64>) {
        self.video_codecs = video_codecs.map(
            |video_codecs| detect_video_codecs(video_codecs as u64 as u8)
        );
    }

    /// Sets the audio codecs.
    ///
    /// # Parameters
    ///
    /// * `audio_codecs: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// This indicates supported audio codecs.
    /// This can specify plural codecs because will be used as a 64 bits flag.
    /// If you set no value, pass the `None`.
    /// See the `AudioCodec` for more detail about the audio codecs flag.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    /// [`AudioCodec`]: ./enum.AudioCodec.html
    pub fn set_audio_codecs(&mut self, audio_codecs: Option<f64>) {
        self.audio_codecs = audio_codecs.map(
            |audio_codecs| detect_audio_codecs(audio_codecs as u64 as u16)
        );
    }

    /// Sets the application name.
    ///
    /// # Parameters
    ///
    /// * `app: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// This can be "". (that is, an empty string)
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_app(&mut self, app: Option<String>) {
        self.app = app;
    }

    /// Sets the command type.
    ///
    /// # Parameters
    ///
    /// * `command_type: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_command_type(&mut self, command_type: Option<String>) {
        self.command_type = command_type;
    }

    /// Sets the Flash Player version.
    ///
    /// # Parameters
    ///
    /// * `flash_ver: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_flash_ver(&mut self, flash_ver: Option<String>) {
        self.flash_ver = flash_ver;
    }

    /// Sets the URL of the video player application connecting with the client.
    ///
    /// # Parameters
    ///
    /// * `swf_url: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_swf_url(&mut self, swf_url: Option<String>) {
        self.swf_url = swf_url;
    }

    /// Sets the URL of the server.
    ///
    /// # Parameters
    ///
    /// * `tc_url: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_tc_url(&mut self, tc_url: Option<String>) {
        self.tc_url = tc_url;
    }

    /// Sets the URL of the web page where the video player application was loaded.
    ///
    /// # Parameters
    ///
    /// * `page_url: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_page_url(&mut self, page_url: Option<String>) {
        self.page_url = page_url;
    }
}

impl From<HashMap<String, AmfData>> for CommandObject {
    /// Converts the `HashMap<String, AmfData>` value into this struct.
    ///
    /// # Parameters
    ///
    /// * `m: HashMap<String, AmfData>`
    ///
    /// A `HashMap<String, AmfData>` value converted from AMF's `Object`.
    /// The `CommandObject` will be sent as the AMF's `Object` type from the client.
    /// This parameter is expected to contain fields of the `CommandObject`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sheave::messages::{
    ///     AmfData,
    ///     CommandObject
    /// };
    ///
    /// let mut m: HashMap<String, AmfData> = HashMap::new();
    ///
    /// m.insert("app".to_string(), AmfData::String("example".to_string()));
    /// m.insert("type".to_string(), AmfData::String("nonprivate".to_string()));
    /// m.insert("flashVer".to_string(), AmfData::String("LNX 9,0,124,2".to_string()));
    /// m.insert("tcUrl".to_string(), AmfData::String("rtmp://example.com/example".to_string()));
    ///
    /// let command_object: CommandObject = m.into();
    ///
    /// /*
    ///  * This will print following format:
    ///  * CommandObject {
    ///  *     fpad: None,
    ///  *     object_encoding: None,
    ///  *     video_function: None,
    ///  *     video_codecs: None,
    ///  *     audio_codecs: None,
    ///  *     app: Some("example"),
    ///  *     command_type: Some("nonprivate"),
    ///  *     flash_ver: Some("LNX 9,0,124,2"),
    ///  *     swf_url: None,
    ///  *     tc_url: Some("rtmp://example.com/example"),
    ///  *     page_url: None
    ///  * }
    /// */
    /// println!("{:?}", command_object);
    /// ```
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
                command_object.set_video_codecs(value.number());
            } else if key == "audioCodecs" {
                command_object.set_audio_codecs(value.number());
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
                println!("Unknown command object: key {}, value {:?}", key, value);
            }
        }

        command_object
    }
}

impl From<CommandObject> for HashMap<String, AmfData> {
    /// Converts this struct into the `HashMap<String, AmfData>`.
    ///
    /// # Parameters
    ///
    /// * `command_object: CommandObject`
    ///
    /// The `CommandObject`'s value.
    ///
    /// This will insert just the value of `Some(...)` into the HashMap.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sheave::messages::{
    ///     AmfData,
    ///     CommandObject
    /// };
    ///
    /// let mut command_object = CommandObject::new();
    ///
    /// command_object.set_app(Some("example".to_string()));
    /// command_object.set_command_type(Some("nonprivate".to_string()));
    /// command_object.set_flash_ver(Some("LNX 9,0,124,2".to_string()));
    /// command_object.set_tc_url(Some("rtmp://example.com/example".to_string()));
    ///
    /// let m: HashMap<String, AmfData> = command_object.into();
    ///
    /// /*
    ///  * This will print following format:
    ///  * {
    ///  *     "app": String("example"),
    ///  *     "type": String("nonprivate"),
    ///  *     "flashVer": String("LNX 9,0,124,2"),
    ///  *     "tcUrl": String("rtmp://example.com/example")
    ///  * }
    /// */
    /// println!("{:?}", m);
    /// ```
    fn from(command_object: CommandObject) -> Self {
        match command_object {
            CommandObject {
                fpad,
                object_encoding,
                video_function,
                video_codecs,
                audio_codecs,
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
                audio_codecs.map(|audio_codecs| m.insert("audioCodecs".to_string(), AmfData::Number(f64::from_bits(convert_audio_codecs_into_flags(audio_codecs) as u64))));
                video_codecs.map(|video_codecs| m.insert("videoCodecs".to_string(), AmfData::Number(f64::from_bits(convert_video_codecs_into_flags(video_codecs) as u64))));
                video_function.map(|video_function| m.insert("videoFunction".to_string(), AmfData::Number(f64::from_bits(video_function as u8 as u64))));
                page_url.map(|page_url| m.insert("pageUrl".to_string(), AmfData::String(page_url)));
                object_encoding.map(|object_encoding| m.insert("objectEncoding".to_string(), AmfData::Number(f64::from_bits(object_encoding as u8 as u64))));
                m
            }
        }
    }
}

/// # The result of the NetConnection command.
///
/// This consists of following patterns:
///
/// * `Result`
/// * `Error`
///
/// This enum and the `String` type can convert into each other because this has implemented the `From<String>` and the `From<NetConnectionResult>`.
/// The correspondence of this enum to actual string result is following:
///
/// |NetConnectionResult|Actual string|
/// | :---------------- | :---------- |
/// |`Result`           |\_result     |
/// |`Error`            |\_error      |
#[derive(Debug, Clone, Copy)]
pub enum NetConnectionResult {
    Result,
    Error
}

impl From<String> for NetConnectionResult {
    /// Converts the `String` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `command: String`
    ///
    /// The result of the NetConnection command.
    /// i.e. \_result or \_error
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value neither \_result nor \_error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::NetConnectionResult;
    ///
    /// let result: NetConnectionResult = "_result".to_string().into();
    /// let error: NetConnectionResult = "_error".to_string().into();
    ///
    /// /* This will print `Result`. */
    /// println!("{:?}", result);
    /// /* This will print `Error`. */
    /// println!("{:?}", error);
    /// ```
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
    /// Converts this enum into the `String` value.
    ///
    /// # Parameters
    ///
    /// * `result: NetConnectionResult`
    ///
    /// The `NetConnectionResult`'s value.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::NetConnectionResult;
    ///
    /// let result: String = NetConnectionResult::Result.into();
    /// let error: String = NetConnectionResult::Error.into();
    ///
    /// /* This will print "_result". */
    /// println!("{}", result);
    /// /* This will print "_error". */
    /// println!("{}", error);
    /// ```
    fn from(result: NetConnectionResult) -> Self {
        match result {
            NetConnectionResult::Result => "_result".to_string(),
            NetConnectionResult::Error => "_error".to_string()
        }
    }
}

/// # The information lebel for InfoObject
///
/// This consists of following pattern:
///
/// * `Status`
/// * `Warning`
/// * `Error`
///
/// ## Status
///
/// This indicates that its command has completed successfully.
///
/// ## Warning
///
/// This indicates that its command has succeeded but has some notice.
///
/// ## Error
///
/// This indicates that its command hasn't succeeded.
///
/// This enum and the `String` type can convert into each other because this has implemented the `From<String>` and the `From<InfoLevel>`.
#[derive(Debug, Clone, Copy)]
pub enum InfoLevel {
    Status,
    Warning,
    Error
}

impl From<String> for InfoLevel {
    /// Converts the `String` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `s: String`
    ///
    /// The string to mean the information level.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value neither "status", "warning" nor "error".
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::InfoLevel;
    ///
    /// let status: InfoLevel = "status".to_string().into();
    /// let warning: InfoLevel = "warning".to_string().into();
    /// let error: InfoLevel = "error".to_string().into();
    ///
    /// /* This will print `Status`. */
    /// println!("{:?}", status);
    /// /* This will print `Warning`. */
    /// println!("{:?}", warning);
    /// /* This will print `Error`. */
    /// println!("{:?}", error);
    /// ```
    fn from(s: String) -> Self {
        use InfoLevel::*;

        if s == "status" {
            Status
        } else if s == "warning" {
            Warning
        } else if s == "error" {
            Error
        } else {
            panic!("Undefined info level!");
        }
    }
}

impl From<InfoLevel> for String {
    /// Converts this enum into the `String` value.
    ///
    /// # Parameters
    ///
    /// * `level: InfoLevel`
    ///
    /// The InfoLevel's value.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::InfoLevel;
    ///
    /// let status: String = InfoLevel::Status.into();
    /// let warning: String = InfoLevel::Warning.into();
    /// let error: String = InfoLevel::Error.into();
    ///
    /// /* This will print "status". */
    /// println!("{}", status);
    /// /* This will print "warning". */
    /// println!("{}", warning);
    /// /* This will print "error". */
    /// println!("{}", error);
    /// ```
    fn from(level: InfoLevel) -> Self {
        use InfoLevel::*;

        match level {
            Status => "status".to_string(),
            Warning => "warning".to_string(),
            Error => "error".to_string()
        }
    }
}

/// # The status of the connect of the NetConnection command
///
/// This consists of following patterns currently:
///
/// * `Success`
///
/// ## Success
///
/// This is used when the connect invoking completed successfully.
///
/// This enum and the `String` type can convert into each other because this has implemented the `From<String>` and the `From<ConnectStatus>`.
/// The correspondence of this enum to actual string is followng:
///
/// |ConnectStatus|Actual string|
/// | :---------- | :---------- |
/// |`Success`    |Success      |
#[derive(Debug, Clone, Copy)]
pub enum ConnectStatus {
    Success
}

impl From<String> for ConnectStatus {
    /// Converts the `String` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `connect_status: String`
    ///
    /// The status of the connect of the NetStream command.
    /// i.e. (NetConnection.Connect.)Success
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the string not to strat with "Success".
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::ConnectStatus;
    ///
    /// let success: ConnectStatus = "Success".to_string().into();
    ///
    /// /* This will print `Success`. */
    /// println!("{:?}", success);
    /// ```
    fn from(connect_status: String) -> Self {
        use ConnectStatus::*;

        if connect_status.starts_with("Success") {
            Success
        } else {
            panic!("Undefined connect status!")
        }
    }
}

impl From<ConnectStatus> for String {
    /// Converts this enum into the `String` value.
    ///
    /// # Parameters
    ///
    /// * `connect_status: ConnectStatus`
    ///
    /// The `ConnectStatus`'s value.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::ConnectStatus;
    ///
    /// let success: String = ConnectStatus::Success.into();
    ///
    /// /* This will print "Success". */
    /// println!("{}", success);
    /// ```
    fn from(connect_status: ConnectStatus) -> Self {
        use ConnectStatus::*;

        match connect_status {
            Success => "Success".to_string()
        }
    }
}

/// # The status of the NetConnection command
///
/// This consists of following patterns currently:
///
/// * `Connect`
///
/// ## Connect
///
/// See the `ConnectStatus`
///
/// This enum and the `String` type can convert into each other because this has implemented the `From<String>` and the `From<NetConnectionStatus>`.
/// The correspondence of this enum to actual string is following:
///
/// |NetConnectionStatus|Actual string|
/// | :---------------- | :---------- |
/// |`Connect(*)`       |Connect.\*   |
///
/// [`ConnectStatus`]: ./enum.ConnectStatus.html
#[derive(Debug, Clone, Copy)]
pub enum NetConnectionStatus {
    Connect(ConnectStatus)
}

impl From<String> for NetConnectionStatus {
    /// Converts the `String` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `net_connection_status: String`
    ///
    /// The status of the NetConnection command.
    /// i.e. (NetConnection.)Connect.\*
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the string not to start with "Connect".
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::NetConnectionStatus;
    ///
    /// let connect: NetConnectionStatus = "Connect.Success".to_string().into();
    ///
    /// /* This will print `Connect(Success)`. */
    /// println!("{:?}", connect);
    /// ```
    fn from(net_connection_status: String) -> Self {
        use NetConnectionStatus::*;

        if net_connection_status.starts_with("Connect") {
            Connect(net_connection_status[(net_connection_status.find(".").unwrap() + 1)..].to_string().into())
        } else {
            panic!("Undefined netconnection status!")
        }
    }
}

impl From<NetConnectionStatus> for String {
    /// Converts this enum into the `String` value.
    ///
    /// # Parameters
    ///
    /// * `net_connection_status: NetConnectionStatus`
    ///
    /// The `NetConnectionStatus`'s value.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::{
    ///     ConnectStatus,
    ///     NetConnectionStatus
    /// };
    ///
    /// let connect: String = NetConnectionStatus::Connect(ConnectStatus::Success).into();
    ///
    /// /* This will print "Connect.Success". */
    /// println!("{}", connect);
    /// ```
    fn from(net_connection_status: NetConnectionStatus) -> Self {
        use NetConnectionStatus::*;

        match net_connection_status {
            Connect(connect_status) => {
                let cs: String = connect_status.into();

                "Connect.".to_string() + cs.as_str()
            }
        }
    }
}

/// # The status of the publish of the NetStream command
///
/// This consists of following patterns currently:
///
/// * `Start`
///
/// ## Start
///
/// This is used when the publish invoking completed successfully.
///
/// This enum and the `String` type can convert into each other because this has implemented the `From<String>` and the `From<PublishStatus>`.
/// The correspondence of this enum to actual string is following:
///
/// |PublishStatus|Actual string|
/// | :---------- | :---------- |
/// |`Start`      |Start        |
#[derive(Debug, Clone, Copy)]
pub enum PublishStatus {
    Start
}

impl From<String> for PublishStatus {
    /// Converts the `String` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `publish_status: String`
    ///
    /// The status of the publish of the NetStream command.
    /// i.e. (NetStream.Publish.)Start
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the string not to start with "Start".
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::PublishStatus;
    ///
    /// let start: PublishStatus = "Start".to_string().into();
    ///
    /// /* This will print `Start`. */
    /// println!("{:?}", start);
    /// ```
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
    /// Converts this enum into the `String` value.
    ///
    /// # Parameters
    ///
    /// * `publish_status: PublishStatus`
    ///
    /// The `PublishStatus`'s value.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::PublishStatus;
    ///
    /// let start: String = PublishStatus::Start.into();
    ///
    /// /* This will print "Start". */
    /// println!("{}", start);
    /// ```
    fn from(publish_status: PublishStatus) -> Self {
        use PublishStatus::*;

        match publish_status {
            Start => "Start".to_string()
        }
    }
}

/// # The status of the NetStream command
///
/// This consists of following patterns currently:
///
/// * `Publish`
///
/// ## Publish
///
/// See the `PublishStatus`.
///
/// This enum and the `String` type can convert into each other because this has implemented the `From<String>` and the `From<NetStreamStatus>`.
/// The correspondence of this enum to actual string is following:
///
/// |NetStreamStatus|Actual string|
/// | :------------ | :---------- |
/// |`Publish(*)`   |Publish.\*   |
///
/// [`PublishStatus`]: ./enum.PublishStatus.html
#[derive(Debug, Clone, Copy)]
pub enum NetStreamStatus {
    Publish(PublishStatus)
}

impl From<String> for NetStreamStatus {
    /// Converts the `String` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `net_stream_status: String`
    ///
    /// The status of the NetStream command.
    /// i.e. (NetStream.)Publish.\*
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the string not to start with "Publish".
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::NetStreamStatus;
    ///
    /// let publish: NetStreamStatus = "Publish.Start".to_string().into();
    ///
    /// /* This will print `Publish(Start)`. */
    /// println!("{:?}", publish);
    /// ```
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
    /// Converts this enum into the `String` value.
    ///
    /// # Parameters
    ///
    /// * `net_stream_status: NetStreamStatus`
    ///
    /// The `NetStreamStatus`'s value.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::{
    ///     NetStreamStatus,
    ///     PublishStatus
    /// };
    ///
    /// let publish: String = NetStreamStatus::Publish(PublishStatus::Start).into();
    ///
    /// /* This will print "Publish.Start". */
    /// println!("{}", publish);
    /// ```
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

/// # The status code
///
/// This consists of following patterns currently:
///
/// * `NetConnection`
/// * `NetStream`
///
/// ## NetConnection
///
/// See the `NetConnectionStatus`.
///
/// ## NetStream
///
/// See the `NetStreamStatus`.
///
/// This enum and the `String` type can convert into each other because this has implemented the `From<String>` and the `From<Status>`.
/// The correspondence of this enum to actual string is following:
///
/// |Status            |Actual string   |
/// | :--------------- | :------------- |
/// |`NetConnection(*)`|NetConnection.\*|
/// |`NetStream(*)`    |NetStream.\*    |
///
/// [`NetConnectionStatus`]: ./enum.NetConnectionStatus.html
/// [`NetStreamStatus`]: ./enum.NetStreamStatus.html
#[derive(Debug, Clone, Copy)]
pub enum Status {
    NetConnection(NetConnectionStatus),
    NetStream(NetStreamStatus)
}

impl From<String> for Status {
    /// Converts the `String` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `status: String`
    ///
    /// The status of the invocation message.
    /// i.e. NetConnection.\* or NetStream.\*
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the string neither to start with "NetConnection" nor to start with "NetStream".
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::Status;
    ///
    /// let net_connection: Status = "NetConnection.Connect.Success".to_string().into();
    /// let net_stream: Status = "NetStream.Publish.Start".to_string().into();
    ///
    /// /* This will print `NetConnection(Connect(Success))`. */
    /// println!("{:?}", net_connection);
    /// /* This will print `NetStream(Publish(Start))`. */
    /// println!("{:?}", net_stream);
    /// ```
    fn from(status: String) -> Self {
        use Status::*;

        if status.starts_with("NetConnection") {
            NetConnection(status[(status.find(".").unwrap() + 1)..].to_string().into())
        } else if status.starts_with("NetStream") {
            NetStream(status[(status.find(".").unwrap() + 1)..].to_string().into())
        } else {
            panic!("Undefined status!")
        }
    }
}

impl From<Status> for String {
    /// Converts this enum into the `String` value.
    ///
    /// # Parameters
    ///
    /// * `status: Status`
    ///
    /// The `Status`'s value.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::{
    ///     ConnectStatus,
    ///     NetConnectionStatus,
    ///     NetStreamStatus,
    ///     PublishStatus,
    ///     Status
    /// };
    ///
    /// let net_connection: String = Status::NetConnection(NetConnectionStatus::Connect(ConnectStatus::Success)).into();
    /// let net_stream: String = Status::NetStream(NetStreamStatus::Publish(PublishStatus::Start)).into();
    ///
    /// /* This will print "NetConnection.Connect.Success". */
    /// println!("{}", net_connection);
    /// /* This will print "NetStream.Publish.Start". */
    /// println!("{}", net_stream);
    /// ```
    fn from(status: Status) -> Self {
        use Status::*;

        match status {
            NetConnection(net_connection_status) => {
                let ncs: String = net_connection_status.into();

                "NetConnection.".to_string() + ncs.as_str()
            },
            NetStream(net_stream_status) => {
                let nss: String = net_stream_status.into();

                "NetStream.".to_string() + nss.as_str()
            }
        }
    }
}

/// # The information object
///
/// This consists of following data currently:
///
/// |Field           |Type                    |
/// | :------------- | :--------------------- |
/// |object\_encoding|`Option<ObjectEncoding>`|
/// |code            |`Option<Status>`        |
/// |level           |`Option<String>`        |
/// |details         |`Option<String>`        |
/// |description     |`Option<String>`        |
///
/// This won't also be contained all field necessarily as the `CommandObject` type is so.
///
/// In common:
///
/// * code
/// * level
/// * description
///
/// In just the result of the connect of the NetConnection command:
///
/// * object\_encoding
///
/// In just the onStatus of the connect of the NetStream command:
///
/// * details
///
/// This type and the `HashMap<String, AmfData>` type can convert into each other because this has implemented the `From<HashMap<String, AmfData>>` and the `From<InfoObject>`
#[derive(Debug, Clone, Default)]
pub struct InfoObject {
    object_encoding: Option<ObjectEncoding>,
    level: Option<InfoLevel>,
    code: Option<Status>,
    details: Option<String>,
    description: Option<String>
}

impl InfoObject {
    /// Constructs a new `InfoObject`.
    pub fn new() -> Self {
        InfoObject {
            object_encoding: None,
            level: None,
            code: None,
            details: None,
            description: None
        }
    }

    /// Sets the object encoding.
    ///
    /// # Parameters
    ///
    /// * `object_encoding: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// This indicates the AMF version which its user uses.
    /// If you set no value, pass the `None`.
    /// See the `ObjectEncoding` for more detail about the object encoding.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_object_encoding(&mut self, object_encoding: Option<f64>) {
        self.object_encoding = object_encoding.map(|object_encoding| (object_encoding as u64 as u8).into());
    }

    /// Sets the level.
    ///
    /// # Parameters
    ///
    /// * `level: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// This indicates the information level which the server will respond.
    /// You will be required to pass one of following values:
    ///
    /// * status
    /// * warning
    /// * error
    ///
    /// If you set no value, pass the `None`.
    /// See the `InfoLevel` for more detail about the level.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    /// [`InfoLevel`]: ./enum.InfoLevel.html
    pub fn set_level(&mut self, level: Option<String>) {
        self.level = level.map(|level| level.into());
    }

    /// Sets the code.
    ///
    /// # Parameters
    ///
    /// * `code: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// This indicates the status code which the server will respond.
    /// You will be required to pass one of following values:
    ///
    /// * NetConnection
    ///   * Connect
    ///     * Success
    /// * NetStream
    ///   * Publish
    ///     * Start
    ///
    /// i.e. Either "NetConnection.Connect.Success" or "NetStream.Publish.Start".
    /// See the `Status` for more detail about the code.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    /// [`Status`]: ./enum.Status.html
    pub fn set_code(&mut self, code: Option<String>) {
        self.code = code.map(|code| code.into());
    }

    /// Sets the details.
    ///
    /// # Parameters
    ///
    /// * `details: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_details(&mut self, details: Option<String>) {
        self.details = details;
    }

    /// Sets the description.
    ///
    /// # Parameters
    ///
    /// * `description: Option<String>`
    ///
    /// A `String` value converted from AMF's string (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }
}

impl From<HashMap<String, AmfData>> for InfoObject {
    /// Converts the `HashMap<String, AmfData>` value into this struct.
    ///
    /// # Parameters
    ///
    /// * `m: HashMap<String, AmfData>`
    ///
    /// The hash map to be converted from the AMF's value.
    /// The `InfoObject` will be sent as the AMF's `Object` type from the client.
    /// This parameter is expected to contain fields of the `InfoObject`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sheave::messages::{
    ///     AmfData,
    ///     ConnectStatus,
    ///     InfoLevel,
    ///     InfoObject,
    ///     NetConnectionStatus,
    ///     Status
    /// };
    ///
    /// let mut m: HashMap<String, AmfData> = HashMap::new();
    ///
    /// m.insert("level".to_string(), AmfData::String(InfoLevel::Status.into()));
    /// m.insert("code".to_string(), AmfData::String(Status::NetConnection(NetConnectionStatus::Connect(ConnectStatus::Success)).into()));
    /// m.insert("description".to_string(), AmfData::String("Connection succeeded.".to_string()));
    /// m.insert("objectEncoding".to_string(), AmfData::Number(0 as f64));
    ///
    /// let info_object: InfoObject = m.into();
    ///
    /// /*
    ///  * This will print following format:
    ///  * InfoObject {
    ///  *     object_encoding: Some(Amf0),
    ///  *     level: Some(Status),
    ///  *     code: Some(NetConnection(Connect(Success))),
    ///  *     details: None,
    ///  *     description: Some("Connection succeeded.")
    ///  * }
    /// */
    /// println!("{:?}", info_object);
    /// ```
    fn from(m: HashMap<String, AmfData>) -> Self {
        let mut info_object = InfoObject::new();

        for (key, value) in m {
            if key == "objectEncoding" {
                info_object.set_object_encoding(value.number());
            } else if key == "level" {
                info_object.set_level(value.string());
            } else if key == "code" {
                info_object.set_code(value.string());
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
    /// Converts this enum into the `HashMap<String, AmfData>` value.
    ///
    /// # Parameters
    ///
    /// * `info_object: InfoObject`
    ///
    /// The `InfoObject`'s value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sheave::messages::{
    ///     AmfData,
    ///     InfoObject
    /// };
    ///
    /// let mut info_object = InfoObject::new();
    ///
    /// info_object.set_level(Some("status".to_string()));
    /// info_object.set_code(Some("NetConnection.Connect.Success".to_string()));
    /// info_object.set_description(Some("Connection succeeded.".to_string()));
    /// info_object.set_object_encoding(Some(0 as f64));
    ///
    /// let m: HashMap<String, AmfData> = info_object.into();
    ///
    /// /*
    ///  * This will print following format:
    ///  * {
    ///  *     "level": String("status"),
    ///  *     "code": String("NetConnection.Connect.Success"),
    ///  *     "description": String("Connection succeeded."),
    ///  *     "objectEncoding": Number(0)
    ///  * }
    /// */
    /// println!("{:?}", m);
    /// ```
    fn from(info_object: InfoObject) -> Self {
        match info_object {
            InfoObject {
                object_encoding,
                level,
                code,
                details,
                description
            } => {
                let mut m: HashMap<String, AmfData> = HashMap::new();

                object_encoding.map(|object_encoding| m.insert("objectEncoding".to_string(), AmfData::Number(object_encoding as u8 as u64 as f64)));
                level.map(|level| m.insert("level".to_string(), AmfData::String(level.into())));
                code.map(|code| m.insert("code".to_string(), AmfData::String(code.into())));
                details.map(|details| m.insert("details".to_string(), AmfData::String(details)));
                description.map(|description| m.insert("description".to_string(), AmfData::String(description)));
                m
            }
        }
    }
}

/// # The NetConnection command.
///
/// This consists of following patterns:
///
/// * `Connect`
/// * `ConnectResult`
/// * `ReleaseStream`
/// * `ReleaseStreamResult`
/// * `CreateStream`
/// * `CreateStreamResult`
///
/// ## Connect
///
/// This consists of following data:
///
/// |Field           |Type              |
/// | :------------- | :--------------- |
/// |transactionn\_id|`u64`             |
/// |command\_object |`CommandObject`   |
/// |argument        |`Option<Argument>`|
///
/// ### transaction\_id
///
/// The number to indicate the step of current invoking.
/// This is converted from the AMF's Number types.
///
/// ### command\_object
///
/// See the `CommandObject`.
///
/// ### argument
///
/// Currently, this will be the None inevitably.
/// This has been defined in the specification but hasn't been able to check its existence yet.
/// Because of the above causes, the `Argument` type has defined like the unit and has been contained in the `Option` type currently.
///
/// ## ConnectResult
///
/// This consists of following data:
///
/// |Field          |Type                      |
/// | :------------ | :----------------------- |
/// |result         |`NetConnectionResult`     |
/// |transaction\_id|`u64`                     |
/// |properties     |`HashMap<String, AmfData>`|
/// |information    |`InfoObject`              |
///
/// ### result
///
/// See the `NetConnectionResult`.
///
/// ### transaction\_id
///
/// This is the same value as the Connect's one.
///
/// ### properties
///
/// The HashMap of `AmfData` associated with the property name.
///
/// ### information
///
/// See the `InfoObject`.
///
/// ## ReleaseStream
///
/// This consists of following data:
///
/// |Field          |Type    |
/// | :------------ | :----- |
/// |transaction\_id|`u64`   |
/// |play\_path     |`String`|
///
/// ### transaction\_id
///
/// This is the same format as the Connect's one.
///
/// ### play\_path
///
/// A part of the URL to be requested from the client when the TCP connection.
/// i.e. *playpath* in rtmp://example.com/appName/playpath
///
/// This can start with the prefix of "mp4:", etc.
///
/// ## ReleaseStreamResult
///
/// This consists of following data:
///
/// |Field          |Type                 |
/// | :------------ | :------------------ |
/// |result         |`NetConnectionResult`|
/// |transaction\_id|`u64`                |
///
/// ### result
///
/// See the `NetConnectionResult`.
///
/// ### transaction\_id
///
/// This is the same value as the ReleaseStream's one.
///
/// ## CreateStream
///
/// This consists of following data:
///
/// |Field          |Type |
/// | :------------ | :-- |
/// |transaction\_id|`u64`|
///
/// ### transaction\_id
///
/// This is the same format as the ReleaseStream's one.
///
/// ## CreateStreamResult
///
/// This consists of following data:
///
/// |Field          |Type                 |
/// | :------------ | :------------------ |
/// |result         |`NetConnectionResult`|
/// |transaction\_id|`u64`                |
/// |message\_id    |`u32`                |
///
/// ### result
///
/// See the `NetConnectionResult`.
///
/// ### transaction\_id
///
/// The same value as the CreateStream's one.
///
/// ### message\_id
///
/// The number to identify its user.
/// This will be used in following parts:
///
/// * The chunk message header's message stream id
/// * The ping's event data
/// * Or some implementations to identify the user
///
/// [`CommandObject`]: ./struct.CommandObject.html
/// [`NetConnectionResult`]: ./struct.NetConnectionResult.html
/// [`InfoObject`]: ./struct.InfoObject.html
#[derive(Debug, Clone)]
pub enum NetConnectionCommand {
    Connect {
        argument: Option<Argument>,
        transaction_id: u64,
        command_object: CommandObject
    },
    ConnectResult {
        result: NetConnectionResult,
        transaction_id: u64,
        properties: HashMap<String, AmfData>,
        information: InfoObject
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

/// # The FCPublish command
///
/// This consists of following patterns:
///
/// * `FcPublish`
/// * `OnFcPublish`
///
/// ## FcPublish
///
/// This consists of following data:
///
/// |Field          |Type    |
/// | :------------ | :----- |
/// |transaction\_id|`u64`   |
/// |play\_path     |`String`|
///
/// ### transaction\_id
///
/// The number to indicate the step of current invoking.
/// This is converted from the AMF's Number types.
///
/// ### play\_path
///
/// A part of the URL to be sent from the client when the TCP connection.
/// i.e. *playpath* in rtmp://example.com/appName/playpath
///
/// This can start with the prefix of "mp4:", etc.
///
/// ## OnFcPublish
///
/// This has no value.
#[derive(Debug, Clone)]
pub enum FcPublishCommand {
    FcPublish {
        transaction_id: u64,
        play_path: String
    },
    OnFcPublish
}

/// # The publish pattern
///
/// This consists of following patterns:
///
/// * `Live`
/// * `Record`
/// * `Append`
///
/// This enum and the `String` type can convert into each other because this has implemented the `From<String>` and the `From<PlayType>`.
/// The correspondence of this enum to actual string is following:
///
/// |PlayType|Actual string|
/// | :----- | :---------- |
/// |`Live`  |live         |
/// |`Record`|record       |
/// |`Append`|append       |
#[derive(Debug, Clone, Copy)]
pub enum PlayType {
    Live,
    Record,
    Append
}

impl From<String> for PlayType {
    /// Converts the `String` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `s: String`
    ///
    /// The play(publish)ing type for its stream.
    /// i.e. "live", "record" or "append"
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the string neither "live", "record" nor "append".
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::PlayType;
    ///
    /// let live: PlayType = "live".to_string().into();
    /// let record: PlayType = "record".to_string().into();
    /// let append: PlayType = "append".to_string().into();
    ///
    /// /* This will print `Live`. */
    /// println!("{:?}", live);
    /// /* This will print `Record`. */
    /// println!("{:?}", record);
    /// /* This will print `Append`. */
    /// println!("{:?}", append);
    /// ```
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
    /// Converts this enum into the `String` value.
    ///
    /// # Parameters
    ///
    /// * `play_type: PlayType`
    ///
    /// The `PlayType`'s value.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::PlayType;
    ///
    /// let live: String = PlayType::Live.into();
    /// let record: String = PlayType::Record.into();
    /// let append: String = PlayType::Append.into();
    ///
    /// /* This will print "live". */
    /// println!("{}", live);
    /// /* This will print "record". */
    /// println!("{}", record);
    /// /* This will print "append". */
    /// println!("{}", append);
    /// ```
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
    /// Constructs a new `PlayType` with its default value, for constructing a new `RtmpHandler`.
    fn default() -> Self {
        PlayType::Live
    }
}

/// # The NetStream command
///
/// This consists of following patterns currently:
///
/// * `Publish`
/// * `OnStatus`
///
/// ## Publish
///
/// This consists of following data:
///
/// |Field          |Type      |
/// | :------------ | :------- |
/// |transaction\_id|`u64`     |
/// |play\_path     |`String`  |
/// |play\_type     |`PlayType`|
///
/// ### transaction\_id
///
/// The number to indicate the step of current invoking.
/// This is converted from the AMF's Number types.
///
/// ### play\_path
///
/// A part of the URL to be requested from the client when the TCP connection.
/// i.e. *playpath* in rtmp://example.com/appName/playpath
///
/// This can start with the prefix of "mp4:", etc.
///
/// ### play\_type
///
/// See the `PlayType`.
///
/// ## OnStatus
///
/// This consists of following data:
///
/// |Field          |Type        |
/// | :------------ | :--------- |
/// |transaction\_id|`u64`       |
/// |info\_object   |`InfoObject`|
///
/// ### transaction\_id
///
/// This is the same value as the Publish's one.
///
/// ### info\_object
///
/// See the `InfoObject`.
///
/// [`PlayType`]: ./enum.PlayType.html
/// [`InfoObject`]: ./struct.InfoObject.html
#[derive(Debug, Clone)]
pub enum NetStreamCommand {
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

/// # The command of the invoke chunk
///
/// This consists of following patterns currently:
///
/// * `NetConnection`
/// * `NetStream`
/// * `FcPublish`
/// * `Unknown`
///
/// ## NetConnection
///
/// This holds following commands:
///
/// * connect
/// * releaseStream
/// * createStream
///
/// ### connect
///
/// This will contain following data respectively:
///
/// In the client:
///
/// 1. Command name
/// 2. Transaction id
/// 3. Command object
/// 4. Optional argument
///
/// In the server:
///
/// 1. Command name
/// 2. Transaction id
/// 3. Properties
/// 4. Information
///
/// #### Command name
///
/// In the client:
///
/// * connect
///
/// In the server:
///
/// * \_result or \_error
///
/// #### Transaction id
///
/// The number to indicate the step of current invoking.
/// In the official specification paper, this is specified that the server must respond by the same number as what the client sent.
///
/// #### Command object
///
/// In the official specification paper, this is specified to consist of the Object type (in AMF0) contained following data:
///
/// |Name          |Type     |
/// | :----------- | :------ |
/// |app           |`String` |
/// |flashVer      |`String` |
/// |swfUrl        |`String` |
/// |tcUrl         |`String` |
/// |fpad          |`Boolean`|
/// |audioCodecs   |`Number` |
/// |videoCodecs   |`Number` |
/// |videoFunction |`Number` |
/// |pageUrl       |`String` |
/// |objectEncoding|`Number` |
///
/// Note that above name/value pairs will be not input all necessarily.
///
/// ##### app
///
/// A part of URL requested from the client when the TCP connection.
/// i.e. *appName* in rtmp://example.com/appName/playpath
///
/// ##### flashVer
///
/// The version of the Flash Player.
/// e.g. FMLE/3.0 (compatible; &lt;Some tool identifiers&gt;), &lt;Some OS identifiers&gt; &lt;Flash Player version&gt;
///
/// ##### swfUrl
///
/// The URL of the place where the player application exists.
/// e.g. the same value as the tcUrl (in Open Broadcaster Software)
///
/// ##### tcUrl
///
/// The URL to be sent when the client requests the TCP connection.
/// e.g. rtmp://example.com/appName/playpath
///
/// ##### fpad
///
/// The value to indicate whether the proxy is being used.
///
/// ##### audioCodecs
///
/// The value to indicate the audio codec.
/// See the `AudioCodec`.
///
/// ##### videoCodecs
///
/// The value to indicate the video codec.
/// See the `VideoCodec`.
///
/// ##### videoFunction
///
/// The value to indicate the video function.
/// See the `VideoFunction`.
///
/// ##### pageUrl
///
/// The URL of the web page where the player application was loaded.
/// e.g. http://example.com/foo.html
///
/// ##### objectEncoding
///
/// The value to indicate the Action Message Format's version.
/// See the `ObjectEncoding`.
///
/// #### Properties
///
/// This consists of the Object type (in AMF0) contained following data currently:
///
/// |Name        |Type    |
/// | :--------- | :----- |
/// |fmsVer      |`String`|
/// |capabilities|`Number`|
///
/// ##### fmsVer
///
/// The version of the Adobe Media Server.
///
/// ##### capabilities
///
/// Currently, this hasn't been become clear yet except this is input some number.
///
/// #### Information
///
/// This consists of the Object type (in AMF0) contained following data currently:
///
/// |Name          |Type    |
/// | :----------- | :----- |
/// |level         |`String`|
/// |code          |`String`|
/// |description   |`String`|
/// |objectEncoding|`Number`|
///
/// ##### level
///
/// The level is one of following values:
///
/// * status
/// * warning
/// * error
///
/// ##### code
///
/// In this case, the code is NetConnection.Connect.Success.
///
/// ##### description
///
/// In this case, the description is "Connection succeeded".
///
/// ##### objectEncoding
///
/// The same format as the command object's one.
///
/// ### releaseStream
///
/// This will contain following data respectively:
///
/// In the client:
///
/// 1. Command name
/// 2. Transaction id
/// 3. Command object
/// 4. Playpath
///
/// In the server:
///
/// 1. Command name
/// 2. Transaction id
/// 3. Command object
///
/// #### Command name
///
/// In the client:
///
/// * releaseStream
///
/// In the server:
///
/// * \_result or \_error
///
/// #### Transaction id
///
/// This is the same format as the connect command's one.
///
/// #### Command object
///
/// This is the same format as the connect command's one but is input the null.
///
/// #### Playpath
///
/// A part of URL to be requested from the client when the TCP connection.
/// i.e. *playpath* in rtmp://example.com/appName/playpath
///
/// This can start with the prefix of "mp4:", etc.
///
/// ### createStream
///
/// This will contain following data respectively:
///
/// In the client:
///
/// 1. Command name
/// 2. Transaction id
/// 3. Command object
///
/// In the server:
///
/// 1. Command name
/// 2. Transaction id
/// 3. Command object
/// 4. Message stream id
///
/// #### Command name
///
/// In the client:
///
/// * createStream
///
/// In the server:
///
/// * \_result or \_error
///
/// #### Transaction id
///
/// This is the same format as the releaseStream command's one.
///
/// #### Command object
///
/// This is the same format as the releaseStream command's one.
/// This is also the null.
///
/// #### Message stream id
///
/// The number to identify its user.
/// This will be used in following parts (however in this case, this must cast to the u32 value):
///
/// * The chunk message header's message stream id
/// * The ping's event data
/// * Or some implementations to identify the user
///
/// ## NetStream
///
/// This holds following command:
///
/// * publish
///
/// ### publish
///
/// This will contain following data respectively:
///
/// In the client:
///
/// 1. Command name
/// 2. Transaction id
/// 3. Command object
/// 4. Playpath
/// 5. Play type
///
/// In the server:
///
/// 1. Command name
/// 2. Transaction id
/// 3. Command object
/// 4. Information
///
/// #### Command name
///
/// In the client:
///
/// * publish
///
/// In the server:
///
/// * onStatus
///
/// #### Transaction id
///
/// This is the same format as the releaseStream command's one.
///
/// #### Command object
///
/// This is the same format as the releaseStream command's one.
/// This is also the null.
///
/// #### Playpath
///
/// This is the same value as the releaseStream command's one.
///
/// #### Play type
///
/// This consists one of following values:
///
/// * live
/// * record
/// * append
///
/// ##### live
///
/// This means that is just published.
///
/// ##### record
///
/// This means that is published, and is recorded as a new file.
/// If its file has already existed, it will be overwritten.
///
/// ##### append
///
/// This means that is published, and is recorded to append to the same file.
/// If its file hasn't existed yet, it will be created.
///
/// #### Information
///
/// This is the same format as the connect command's one except the details exists instead of the objectEncoding.
/// That is, this consists of following name/value pairs:
///
/// |Name       |Type    |
/// | :-------- | :----- |
/// |level      |`String`|
/// |code       |`String`|
/// |description|`String`|
/// |details    |`String`|
///
/// ##### level
///
/// The level is one of following values:
///
/// * status
/// * warning
/// * error
///
/// ##### code
///
/// In this case, the code is NetStream.Publish.Start.
///
/// ##### description
///
/// In this case, the decription is "*playpath* is now published".
///
/// ##### details
///
/// This is the same value as the playpath.
///
/// ## FCPublish
///
/// This holds following command:
///
/// * FCPublish
///
/// ### FCPublish
///
/// This will contain following data respectively:
///
/// In the client:
///
/// 1. Command name
/// 2. Transaction id
/// 3. Command object
/// 4. Playpath
///
/// In the server:
///
/// 1. Command name
///
/// #### Command name
///
/// In the client:
///
/// * FCPublish
///
/// In the server:
///
/// * onFCPublish
///
/// #### Transaction id
///
/// This is the same format as the releaseStream command's one.
///
/// #### Command object
///
/// This is the same format as the releaseStream command's one.
/// This is also the null.
///
/// #### Playpath
///
/// This is the same value as the releaseStream command's one.
///
/// ## Unknown
///
/// This will be used when its command is undefined.
/// This is just stored all remaining bytes.
///
/// [AMF0]: ./enum.AmfData.html
/// [`AudioCodec`]: ./enum.AudioCodec.html
/// [`VideoCodec`]: ./enum.VideoCodec.html
/// [`VideoFunction`]: ./enum.VideoFunction.html
/// [`ObjectEncoding`]: ./enum.ObjectEncoding.html
#[derive(Debug, Clone)]
pub enum InvokeCommand {
    NetConnection(NetConnectionCommand),
    NetStream(NetStreamCommand),
    FcPublish(FcPublishCommand),
    Unknown(Vec<u8>)
}

impl InvokeCommand {
    /// Returns whether this is the connect of the NetConnection command.
    pub fn is_connect(&self) -> bool {
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

    /// Returns whether this is the releaseStream of the NetConnection command.
    pub fn is_release_stream(&self) -> bool {
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

    /// Returns whether this is the createStream of the NetConnection command.
    pub fn is_create_stream(&self) -> bool {
        match self {
            &InvokeCommand::NetConnection(
                NetConnectionCommand::CreateStream {
                    transaction_id: _
                }
            ) => true,
            _ => false
        }
    }

    /// Returns the reference of the NetConnection command if this is the NetConnection command.
    /// If this isn't the NetConnection command, returns the None.
    pub fn net_connection(&self) -> Option<&NetConnectionCommand> {
        match self {
            &InvokeCommand::NetConnection(ref net_connection_command) => Some(net_connection_command),
            _ => None
        }
    }

    /// Returns whether this is the FCPublish command.
    pub fn is_fc_publish(&self) -> bool {
        match self {
            &InvokeCommand::FcPublish(_) => true,
            _ => false
        }
    }

    /// Returns the FCPublish command if this is the FCPublish command.
    /// If this isn't the FCPublish command, returns the None.
    pub fn fc_publish(&self) -> Option<&FcPublishCommand> {
        match self {
            &InvokeCommand::FcPublish(ref fc_publish_command) => Some(fc_publish_command),
            _ => None
        }
    }

    /// Returns whether this is the publish of the NetStream command.
    pub fn is_publish(&self) -> bool {
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

    /// Returns the NetStream command if this is the NetStream command.
    /// If this isn't the NetStream command, returns the None.
    pub fn net_stream(&self) -> Option<&NetStreamCommand> {
        match self {
            &InvokeCommand::NetStream(ref net_stream_command) => Some(net_stream_command),
            _ => None
        }
    }
}

/// # The client bandwidth limit type
///
/// The correspondence of the number to this enum is following:
///
/// |Number|LimitType|
/// | ---: | :------ |
/// |0     |`Hard`   |
/// |1     |`Soft`   |
/// |2     |`Dynamic`|
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>`, and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LimitType {
    Hard,
    Soft,
    Dynamic
}

impl From<u8> for LimitType {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `limit_type: u8`
    ///
    /// The number to indicate the client side bandwidth's limit type.
    /// i.e. 0, 1 or 2
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the number above 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::LimitType;
    ///
    /// let hard: LimitType = (0 as u8).into();
    /// let soft: LimitType = (1 as u8).into();
    /// let dynamic: LimitType = (2 as u8).into();
    ///
    /// /* This will print `Hard`. */
    /// println!("{:?}", hard);
    /// /* This will print `Soft`. */
    /// println!("{:?}", soft);
    /// /* This will print `Dynamic`. */
    /// println!("{:?}", dynamic);
    /// ```
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
    /// Constructs a new `LimitType` with its default value, for constructing a new `RtmpHandler`.
    fn default() -> Self {
        LimitType::Hard
    }
}

/// # The ping event type
///
/// The correspondence of the number to this enum is following:
///
/// |Number|PingType     |
/// | ---: | :---------- |
/// |0     |`StreamBegin`|
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>`, and has the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PingType {
    StreamBegin
}

impl From<u8> for PingType {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `ping_type: u8`
    ///
    /// The number to indicate the ping's event type.
    /// i.e. 0 (just this has been implemented currently)
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the number above 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::messages::PingType;
    ///
    /// let stream_begin: PingType = (0 as u8).into();
    ///
    /// /* This will print `StreamBegin`. */ 
    /// println!("{:?}", stream_begin);
    /// ```
    fn from(ping_type: u8) -> Self {
        use PingType::*;

        match ping_type {
            0 => StreamBegin,
            _ => panic!("Undefined ping type!")
        }
    }
}

/// # The ping event data
///
/// The correspondence of the event type to this enum is following:
///
/// |EventType    |Event data           |
/// | :---------- | :------------------ |
/// |`StreamBegin`|The message stream id|
#[derive(Debug, Clone)]
pub enum PingData {
    StreamBegin(u32)
}

/// # The metadata of FLV
///
/// This consists of following data:
///
/// |Field              |Type            |
/// | :---------------- | :------------- |
/// |stereo             |`Option<bool>`  |
/// |can\_seek\_to\_end |`Option<bool>`  |
/// |audio\_codec\_id   |`Option<u64>`   |
/// |audio\_data\_rate  |`Option<u64>`   |
/// |audio\_sample\_rate|`Option<u64>`   |
/// |audio\_sample\_size|`Option<u64>`   |
/// |audio\_delay       |`Option<u64>`   |
/// |video\_codec\_id   |`Option<u64>`   |
/// |video\_data\_rate  |`Option<u64>`   |
/// |frame\_rate        |`Option<u64>`   |
/// |width              |`Option<u64>`   |
/// |height             |`Option<u64>`   |
/// |file\_size         |`Option<u64>`   |
/// |duration           |`Option<u64>`   |
/// |major\_brand       |`Option<String>`|
/// |minor\_version     |`Option<String>`|
/// |compatible\_brands |`Option<String>`|
/// |encoder            |`Option<String>`|
/// |creation\_date     |`Option<String>`|
///
/// We can check that the metadata has all value of above.
/// However all field has been contained in the `Option` type because the metadata will be passed as the AMF0's `MixedArray` type, that is, this must be converted from the `HashMap<String, AmfData>`.
/// The number fields have kept their types the `u64` type to consider that will lessen the overhead to convert Rust's number types into AMF's number types.
/// See the flv.rs about more details of above fields.
///
/// This type and the `HashMap<String, AmfData>` type can convert into each other because this has implemented the `From<HashMap<String, AmfData>>` and the `From<MetaData>`.
///
/// [flv.rs]: ../flv.rs.html
#[derive(Debug, Clone)]
pub struct MetaData {
    stereo: Option<bool>,
    can_seek_to_end: Option<bool>,
    audio_codec_id: Option<u64>,
    audio_data_rate: Option<u64>,
    audio_sample_rate: Option<u64>,
    audio_sample_size: Option<u64>,
    audio_delay: Option<u64>,
    video_codec_id: Option<u64>,
    video_data_rate: Option<u64>,
    frame_rate: Option<u64>,
    width: Option<u64>,
    height: Option<u64>,
    file_size: Option<u64>,
    duration: Option<u64>,
    major_brand: Option<String>,
    minor_version: Option<String>,
    compatible_brands: Option<String>,
    encoder: Option<String>,
    creation_date: Option<String>
}

impl MetaData {
    /// Constructs a new `MetaData`.
    pub fn new() -> Self {
        MetaData {
            stereo: None,
            can_seek_to_end: None,
            audio_codec_id: None,
            audio_data_rate: None,
            audio_sample_rate: None,
            audio_sample_size: None,
            audio_delay: None,
            video_codec_id: None,
            video_data_rate: None,
            frame_rate: None,
            width: None,
            height: None,
            file_size: None,
            duration: None,
            major_brand: None,
            minor_version: None,
            compatible_brands: None,
            encoder: None,
            creation_date: None
        }
    }

    /// Returns whether the audio has corresponded stereo.
    pub fn is_stereo(&mut self) -> Option<bool> {
        self.stereo
    }

    /// Sets the stereo.
    ///
    /// # Parameters
    ///
    /// * `stereo: Option<bool>`
    ///
    /// A `bool` value converted from AMF's `Boolean`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Boolean`]: ./enum.AmfData.html#variant.Boolean
    pub fn set_stereo(&mut self, stereo: Option<bool>) {
        self.stereo = stereo;
    }

    /// Retrns whether the video can seek to end.
    pub fn can_seek_to_end(&self) -> Option<bool> {
        self.can_seek_to_end
    }

    /// Sets the canSeekToEnd.
    ///
    /// # Parameters
    ///
    /// * `can_seek_to_end: Option<bool>`
    ///
    /// A `bool` value converted from AMF's `Boolean`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Boolean`]: ./enum.AmfData.html#variant.Boolean
    pub fn set_can_seek_to_end(&mut self, can_seek_to_end: Option<bool>) {
        self.can_seek_to_end = can_seek_to_end;
    }

    /// Returns the audio codec id.
    pub fn get_audio_codec_id(&self) -> Option<u64> {
        self.audio_codec_id
    }

    /// Sets the audio codec id.
    ///
    /// # Parameters
    ///
    /// * `audio_codec_id: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_audio_codec_id(&mut self, audio_codec_id: Option<f64>) {
        self.audio_codec_id = audio_codec_id.map(
            |audio_codec_id| audio_codec_id as u64
        );
    }

    /// Returns the audio data rate.
    pub fn get_audio_data_rate(&self) -> Option<u64> {
        self.audio_data_rate
    }

    /// Sets the audio data rate.
    ///
    /// # Parameters
    ///
    /// * `audio_data_rate: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_audio_data_rate(&mut self, audio_data_rate: Option<f64>) {
        self.audio_data_rate = audio_data_rate.map(
            |audio_data_rate| audio_data_rate as u64
        );
    }

    /// Returns the audio sample rate.
    pub fn get_audio_sample_rate(&self) -> Option<u64> {
        self.audio_sample_rate
    }

    /// Sets the audio sample rate.
    ///
    /// # Parameters
    ///
    /// * `audio_sample_rate: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_audio_sample_rate(&mut self, audio_sample_rate: Option<f64>) {
        self.audio_sample_rate = audio_sample_rate.map(
            |audio_sample_rate| audio_sample_rate as u64
        );
    }

    /// Returns the audio sample size.
    pub fn get_audio_sample_size(&self) -> Option<u64> {
        self.audio_sample_size
    }

    /// Sets the audio sample size.
    ///
    /// # Parameters
    ///
    /// * `audio_sample_size: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_audio_sample_size(&mut self, audio_sample_size: Option<f64>) {
        self.audio_sample_size = audio_sample_size.map(
            |audio_sample_size| audio_sample_size as u64
        );
    }

    /// Returns the audio delay.
    pub fn get_audio_delay(&self) -> Option<u64> {
        self.audio_delay
    }

    /// Sets the audio delay.
    ///
    /// # Parameters
    ///
    /// * `audio_delay: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_audio_delay(&mut self, audio_delay: Option<f64>) {
        self.audio_delay = audio_delay.map(
            |audio_delay| audio_delay as u64
        );
    }

    /// Returns the video codec id.
    pub fn get_video_codec_id(&self) -> Option<u64> {
        self.video_codec_id
    }

    /// Sets the video codec id.
    ///
    /// # Parameters
    ///
    /// * `video_codec_id: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_video_codec_id(&mut self, video_codec_id: Option<f64>) {
        self.video_codec_id = video_codec_id.map(
            |video_codec_id| video_codec_id as u64
        );
    }

    /// Returns the video data rate.
    pub fn get_video_data_rate(&self) -> Option<u64> {
        self.video_data_rate
    }

    /// Sets the video data rate.
    ///
    /// # Parameters
    ///
    /// * `video_data_rate: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_video_data_rate(&mut self, video_data_rate: Option<f64>) {
        self.video_data_rate = video_data_rate.map(
            |video_data_rate| video_data_rate as u64
        );
    }

    /// Returns the frame rate.
    pub fn get_frame_rate(&self) -> Option<u64> {
        self.frame_rate
    }

    /// Sets the frame rate.
    ///
    /// # Parameters
    ///
    /// * `frame_rate: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_frame_rate(&mut self, frame_rate: Option<f64>) {
        self.frame_rate = frame_rate.map(
            |frame_rate| frame_rate as u64
        );
    }

    /// Returns the width.
    pub fn get_width(&self) -> Option<u64> {
        self.width
    }

    /// Sets the width.
    ///
    /// # Parameters
    ///
    /// * `width: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_width(&mut self, width: Option<f64>) {
        self.width = width.map(
            |width| width as u64
        );
    }

    /// Returns the height.
    pub fn get_height(&self) -> Option<u64> {
        self.height
    }

    /// Sets the hiehgt.
    ///
    /// # Parameters
    ///
    /// * `height: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_height(&mut self, height: Option<f64>) {
        self.height = height.map(
            |height| height as u64
        );
    }

    /// Returns the file size.
    pub fn get_file_size(&self) -> Option<u64> {
        self.file_size
    }

    /// Sets the file size.
    ///
    /// # Parameters
    ///
    /// * `file_size: Option<f64>`
    ///
    /// A `f64` value converted from AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_file_size(&mut self, file_size: Option<f64>) {
        self.file_size = file_size.map(
            |file_size| file_size as u64
        );
    }

    /// Returns the duration.
    pub fn get_duration(&self) -> Option<u64> {
        self.duration
    }

    /// Sets the duration.
    ///
    /// # Parameters
    ///
    /// * `duration: Option<f64>`
    ///
    /// A `f64` value converted AMF's `Number`.
    /// If you set no value, pass the `None`.
    ///
    /// [`Number`]: ./enum.AmfData.html#variant.Number
    pub fn set_duration(&mut self, duration: Option<f64>) {
        self.duration = duration.map(
            |duration| duration as u64
        );
    }

    /// Returns the major brand.
    pub fn get_major_brand(&self) -> &Option<String> {
        &self.major_brand
    }

    /// Sets the major brand.
    ///
    /// # Parameters
    ///
    /// * `major_brand: Option<String>`
    ///
    /// A `f64` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_major_brand(&mut self, major_brand: Option<String>) {
        self.major_brand = major_brand;
    }

    /// Returns the minor version.
    pub fn get_minor_version(&self) -> &Option<String> {
        &self.minor_version
    }

    /// Sets the minor version.
    ///
    /// # Parameters
    ///
    /// * `minor_version: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_minor_version(&mut self, minor_version: Option<String>) {
        self.minor_version = minor_version;
    }

    /// Returns the compatible brands.
    pub fn get_compatible_brands(&self) -> &Option<String> {
        &self.compatible_brands
    }

    /// Sets the compatible brands.
    ///
    /// # Parameters
    ///
    /// * `compatible_brands: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_compatible_brands(&mut self, compatible_brands: Option<String>) {
        self.compatible_brands = compatible_brands;
    }

    /// Returns the encoder.
    pub fn get_encoder(&self) -> &Option<String> {
        &self.encoder
    }

    /// Sets the encoder.
    ///
    /// # Parameters
    ///
    /// * `encoder: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_encoder(&mut self, encoder: Option<String>) {
        self.encoder = encoder;
    }

    /// Returns the creation date.
    pub fn get_creation_date(&self) -> &Option<String> {
        &self.creation_date
    }

    /// Sets the creation date.
    ///
    /// # Parameters
    ///
    /// * `creation_date: Option<String>`
    ///
    /// A `String` value converted from AMF's String (`AmfData::String`).
    /// If you set no value, pass the `None`.
    ///
    /// [`AmfData::String`]: ./enum.AmfData.html#variant.String
    pub fn set_creation_date(&mut self, creation_date: Option<String>) {
        self.creation_date = creation_date;
    }
}

impl From<HashMap<String, AmfData>> for MetaData {
    /// Converts the `HashMap<String, AmfData>` value into this struct.
    ///
    /// # Parameters
    ///
    /// * `m: HashMap<String, AmfData>`
    ///
    /// The hash map to be converted from the AMF's value.
    /// The `MetaData` will be sent as the AMF's `MixedArray` type from the client.
    /// This parameter is expected to contain fields of the FLV's metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sheave::messages::{
    ///     AmfData,
    ///     MetaData
    /// };
    ///
    /// let mut m: HashMap<String, AmfData> = HashMap::new();
    ///
    /// m.insert("stereo".to_string(), AmfData::Boolean(true));
    /// m.insert("audiocodecid".to_string(), AmfData::Number(2 as f64));
    /// m.insert("audiodatarate".to_string(), AmfData::Number(320 as f64));
    /// m.insert("audiosamplerate".to_string(), AmfData::Number(44100 as f64));
    /// m.insert("audiosamplesize".to_string(), AmfData::Number(16 as f64));
    /// m.insert("videocodecid".to_string(), AmfData::Number(2 as f64));
    /// m.insert("videodatarate".to_string(), AmfData::Number(3000 as f64));
    /// m.insert("framerate".to_string(), AmfData::Number(30 as f64));
    /// m.insert("width".to_string(), AmfData::Number(1920 as f64));
    /// m.insert("height".to_string(), AmfData::Number(1080 as f64));
    /// m.insert("filesize".to_string(), AmfData::Number(3000000 as f64));
    /// m.insert("duration".to_string(), AmfData::Number(1 as f64));
    /// m.insert("major-brand".to_string(), AmfData::String("".to_string()));
    /// m.insert("minor-version".to_string(), AmfData::String("".to_string()));
    /// m.insert("compatible-brands".to_string(), AmfData::String("".to_string()));
    /// m.insert("encoder".to_string(), AmfData::String("".to_string()));
    ///
    /// let meta_data: MetaData = m.into();
    ///
    /// /*
    ///  * This will print following format:
    ///  * MetaData {
    ///  *     stereo: Some(true),
    ///  *     audio_codec_id: Some(2),
    ///  *     audio_data_rate: Some(320),
    ///  *     audio_sample_rate: Some(44100),
    ///  *     audio_sample_size: Some(16),
    ///  *     video_codec_id: Some(2),
    ///  *     video_data_rate: Some(3000),
    ///  *     frame_rate: Some(30),
    ///  *     width: Some(1920),
    ///  *     height: Some(1080),
    ///  *     file_size: Some(3000),
    ///  *     duration: Some(1),
    ///  *     major_brand: Some(""),
    ///  *     minor_version: Some(""),
    ///  *     compatible_brands: Some(""),
    ///  *     encoder: Some("")
    ///  * }
    /// */
    /// println!("{:?}", meta_data);
    /// ```
    fn from(m: HashMap<String, AmfData>) -> Self {
        let mut meta_data = MetaData::new();

        for (key, value) in m {
            if key == "stereo" {
                meta_data.set_stereo(value.boolean());
            } else if key == "canSeekToEnd" {
                meta_data.set_can_seek_to_end(value.boolean());
            } else if key == "audiocodecid" {
                meta_data.set_audio_codec_id(value.number());
            } else if key == "audiodatarate" {
                meta_data.set_audio_data_rate(value.number());
            } else if key == "audiosamplerate" {
                meta_data.set_audio_sample_rate(value.number());
            } else if key == "audiosamplesize" {
                meta_data.set_audio_sample_size(value.number());
            } else if key == "audiodelay" {
                meta_data.set_audio_delay(value.number());
            } else if key == "videocodecid" {
                meta_data.set_video_codec_id(value.number());
            } else if key == "videodatarate" {
                meta_data.set_video_data_rate(value.number());
            } else if key == "framerate" {
                meta_data.set_frame_rate(value.number());
            } else if key == "width" {
                meta_data.set_width(value.number());
            } else if key == "height" {
                meta_data.set_height(value.number());
            } else if key == "filesize" {
                meta_data.set_file_size(value.number());
            } else if key == "duration" {
                meta_data.set_duration(value.number());
            } else if key == "major-brand" {
                meta_data.set_major_brand(value.string());
            } else if key == "minor-version" {
                meta_data.set_minor_version(value.string());
            } else if key == "compatible-brands" {
                meta_data.set_compatible_brands(value.string());
            } else if key == "encoder" {
                meta_data.set_encoder(value.string());
            } else if key == "creationdate" {
                meta_data.set_creation_date(value.string());
            } else {
                println!("Unknown metadata: key {}, value {:?}", key, value);
            }
        }

        meta_data
    }
}

impl From<MetaData> for HashMap<String, AmfData> {
    /// Converts this struct into the `HashMap<String, AmfData>`
    ///
    /// # Parameters
    ///
    /// * `metadata: MetaData`
    ///
    /// The `MetaData`'s value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sheave::messages::{
    ///     AmfData,
    ///     MetaData
    /// };
    ///
    /// let mut meta_data = MetaData::new();
    ///
    /// meta_data.set_stereo(Some(true));
    /// meta_data.set_audio_codec_id(Some(2 as f64));
    /// meta_data.set_audio_data_rate(Some(320 as f64));
    /// meta_data.set_audio_sample_rate(Some(44100 as f64));
    /// meta_data.set_audio_sample_size(Some(16 as f64));
    /// meta_data.set_video_codec_id(Some(2 as f64));
    /// meta_data.set_video_data_rate(Some(3000 as f64));
    /// meta_data.set_frame_rate(Some(30 as f64));
    /// meta_data.set_width(Some(1920 as f64));
    /// meta_data.set_height(Some(1080 as f64));
    /// meta_data.set_file_size(Some(3000000 as f64));
    /// meta_data.set_duration(Some(1 as f64));
    /// meta_data.set_major_brand(Some("".to_string()));
    /// meta_data.set_minor_version(Some("".to_string()));
    /// meta_data.set_compatible_brands(Some("".to_string()));
    /// meta_data.set_encoder(Some("".to_string()));
    ///
    /// let m: HashMap<String, AmfData> = meta_data.into();
    ///
    /// /*
    ///  * This will print following format:
    ///  * {
    ///  *     "stereo": Boolean(true),
    ///  *     "audiocodecid": Number(2),
    ///  *     "audiodatarate": Number(320),
    ///  *     "audiosamplerate": Number(44100),
    ///  *     "audiosamplesize": Number(16),
    ///  *     "videocodecid": Number(2),
    ///  *     "videodatarate": Number(3000),
    ///  *     "framerate": Number(30),
    ///  *     "width": Number(1920),
    ///  *     "height": Number(1080),
    ///  *     "filesize": Number(3000000),
    ///  *     "duration": Number(1),
    ///  *     "major-brand": String(""),
    ///  *     "minor-version": String(""),
    ///  *     "compatible-brands": String("")
    ///  * }
    /// */
    /// println!("{:?}", m);
    /// ```
    fn from(metadata: MetaData) -> Self {
        let mut m: HashMap<String, AmfData> = HashMap::new();

        match metadata {
            MetaData {
                stereo,
                can_seek_to_end,
                audio_codec_id,
                audio_data_rate,
                audio_sample_rate,
                audio_sample_size,
                audio_delay,
                video_codec_id,
                video_data_rate,
                frame_rate,
                width,
                height,
                file_size,
                duration,
                major_brand,
                minor_version,
                compatible_brands,
                encoder,
                creation_date
            } => {
                stereo.map(
                    |stereo| m.insert("stereo".to_string(), AmfData::Boolean(stereo))
                );
                can_seek_to_end.map(
                    |can_seek_to_end| m.insert("canSeekToEnd".to_string(), AmfData::Boolean(can_seek_to_end))
                );
                audio_codec_id.map(
                    |audio_codec_id| m.insert("audiocodecid".to_string(), AmfData::Number(audio_codec_id as f64))
                );
                audio_data_rate.map(
                    |audio_data_rate| m.insert("audiodatarate".to_string(), AmfData::Number(audio_data_rate as f64))
                );
                audio_sample_rate.map(
                    |audio_sample_rate| m.insert("audiosamplerate".to_string(), AmfData::Number(audio_sample_rate as f64))
                );
                audio_sample_size.map(
                    |audio_sample_size| m.insert("audiosamplesize".to_string(), AmfData::Number(audio_sample_size as f64))
                );
                audio_delay.map(
                    |audio_delay| m.insert("audiodelay".to_string(), AmfData::Number(audio_delay as f64))
                );
                video_codec_id.map(
                    |video_codec_id| m.insert("videocodecid".to_string(), AmfData::Number(video_codec_id as f64))
                );
                video_data_rate.map(
                    |video_data_rate| m.insert("videodatarate".to_string(), AmfData::Number(video_data_rate as f64))
                );
                frame_rate.map(
                    |frame_rate| m.insert("framerate".to_string(), AmfData::Number(frame_rate as f64))
                );
                width.map(
                    |width| m.insert("width".to_string(), AmfData::Number(width as f64))
                );
                height.map(
                    |height| m.insert("height".to_string(), AmfData::Number(height as f64))
                );
                file_size.map(
                    |file_size| m.insert("filesize".to_string(), AmfData::Number(file_size as f64))
                );
                duration.map(
                    |duration| m.insert("duration".to_string(), AmfData::Number(duration as f64))
                );
                major_brand.map(
                    |major_brand| m.insert("major-brand".to_string(), AmfData::String(major_brand))
                );
                minor_version.map(
                    |minor_version| m.insert("minor-version".to_string(), AmfData::String(minor_version))
                );
                compatible_brands.map(
                    |compatible_brands| m.insert("compatible-brands".to_string(), AmfData::String(compatible_brands))
                );
                encoder.map(
                    |encoder| m.insert("encoder".to_string(), AmfData::String(encoder))
                );
                creation_date.map(
                    |creation_date| m.insert("creationdate".to_string(), AmfData::String(creation_date))
                );
            }
        }

        m
    }
}

/// # The notify command
///
/// This consists of following patterns:
///
/// * `SetDataFrame`
/// * `Unknown`
///
/// ## SetDataFrame
///
/// This consists of following data:
///
/// |Field      |Type      |
/// | :-------- | :------- |
/// |data\_frame|`String`  |
/// |meta\_data |`MetaData`|
///
/// ### data\_frame
///
/// This indicates the name of metadata to input to the FLV.
/// The data frame is following currently:
///
/// * onMetaData
///
/// ### meta\_data
///
/// See the `MetaData`.
///
/// ## Unknown
///
/// This will be used when the data frame is undefined.
/// This will be stored all remaining bytes.
///
/// [`MetaData`]: ./struct.MetaData.html
#[derive(Clone, Debug)]
pub enum NotifyCommand {
    SetDataFrame {
        data_frame: String,
        meta_data: MetaData
    },
    Unknown(Vec<u8>)
}

impl NotifyCommand {
    /// Returns whether this is the `SetDataFrame`.
    pub fn is_data_frame(&self) -> bool {
        match self {
            &NotifyCommand::SetDataFrame {
                data_frame: _,
                meta_data: _
            } => true,
            _ => false
        }
    }

    /// Returns the `SetDataFrame` if this is the `SetDataFrame`.
    /// Otherwise returns the `None`
    pub fn data_frame(&self) -> Option<(&String, &MetaData)> {
        match self {
            &NotifyCommand::SetDataFrame {
                ref data_frame,
                ref meta_data
            } => Some((data_frame, meta_data)),
            _ => None
        }
    }
}

/// # The chunk data
///
/// This consists of following patterns:
///
/// 1. Chunk size
/// 2. Bytes read
/// 3. Ping
/// 4. Server bandwidth
/// 5. Client bandwidth
/// 6. Audio
/// 7. Video
/// 8. Notify (AMF0)
/// 9. Invoke (AMF0)
/// 10. Unknown (Other)
///
/// ## Chunk size
///
/// This is the same as messages.rs.
///
/// ## Bytes read
///
/// This is the same as messages.rs.
///
/// ## Ping
///
/// See the `PingData`.
///
/// ## Server bandwidth
///
/// This is the same as messages.rs.
///
/// ## Client bandwidth
///
/// This is the same as messages.rs.
///
/// ## Audio
///
/// This is the same as messages.rs.
///
/// ## Video
///
/// This is the same as messages.rs.
///
/// ## Notify
///
/// See the `NotifyCommand`.
///
/// ## Invoke
///
/// See the `InvokeCommand`.
///
/// [messages.rs]: ../messages.rs.html
/// [`PingData`]: ./enum.PingData.html
/// [`NotifyCommand`]: ./enum.NotifyCommand.html
/// [`InvokeCommand`]: ./enum.InvokeCommand.html
#[derive(Debug, Clone)]
pub enum ChunkData {
    ChunkSize(u32),
    BytesRead(u32),
    Ping(PingData),
    ServerBandwidth(u32),
    ClientBandwidth(u32, LimitType),
    Audio(Vec<u8>),
    Video(Vec<u8>),
    Notify(NotifyCommand),
    Invoke(InvokeCommand),
    Unknown(Vec<u8>)
}

impl ChunkData {
    /// Returns the chunk size if this is the `ChunkSize`, otherwise returns the `None`.
    pub fn chunk_size(&self) -> Option<u32> {
        match self {
            &ChunkData::ChunkSize(chunk_size) => Some(chunk_size),
            _ => None
        }
    }

    /// Returns the bytes read if this is the `BytesRead`, otherwise returns the `None`.
    pub fn bytes_read(&self) -> Option<u32> {
        match self {
            &ChunkData::BytesRead(bytes_read) => Some(bytes_read),
            _ => None
        }
    }

    /// Returns the `PingData` if this is the `Ping`, otherwise returns the `None`.
    pub fn ping(&self) -> Option<&PingData> {
        match self {
            &ChunkData::Ping(ref ping) => Some(ping),
            _ => None
        }
    }

    /// Returns the server side bandwidth if this is the `ServerBandwidth`, otherwise returns the `None`.
    pub fn server_bandwidth(&self) -> Option<u32> {
        match self {
            &ChunkData::ServerBandwidth(bandwidth) => Some(bandwidth),
            _ => None
        }
    }

    /// Returns the client side bandwidth and its limit type if this is the `ClientBandwidth`, otherwise returns the `None`.
    pub fn client_bandwidth(&self) -> Option<(u32, LimitType)> {
        match self {
            &ChunkData::ClientBandwidth(bandwidth, limit_type) => Some((bandwidth, limit_type)),
            _ => None
        }
    }

    /// Returns the audio bytes if this is the `Audio`, otherwise returns the `None`.
    pub fn audio(&self) -> Option<&Vec<u8>> {
        match self {
            &ChunkData::Audio(ref bytes) => Some(bytes),
            _ => None
        }
    }

    /// Returns the audio bytes if this is the `Video`, otherwise returns the `None`.
    pub fn video(&self) -> Option<&Vec<u8>> {
        match self {
            &ChunkData::Video(ref bytes) => Some(bytes),
            _ => None
        }
    }

    /// Returns the `NotifyCommand` if this is the `Notify`, otherwise returns the `None`.
    pub fn notify(&self) -> Option<&NotifyCommand> {
        match self {
            &ChunkData::Notify(ref notify_command) => Some(notify_command),
            _ => None
        }
    }

    /// Returns the `InvokeCommand` if this is the `Invoke`, otherwise returns the `None`.
    pub fn invoke(&self) -> Option<&InvokeCommand> {
        match self {
            &ChunkData::Invoke(ref invoke_command) => Some(invoke_command),
            _ => None
        }
    }

    /// Returns the unknown bytes if this is the `Unknown`, otherwise returns the `None`.
    pub fn unknown(&self) -> Option<&Vec<u8>> {
        match self {
            &ChunkData::Unknown(ref unknown) => Some(unknown),
            _ => None
        }
    }
}

/// # The chunk
///
/// This will be used to store the chunk converted from the bytes.
/// This consists of following data:
///
/// |Field              |Type               |
/// | :---------------- | :---------------- |
/// |basic\_header      |`BasicHeader`      |
/// |extended\_timestamp|`Option<Duration>` |
/// |message\_header    |`MessageHeader`    |
/// |chunk\_data        |`Option<ChunkData>`|
///
/// [`BasicHeader`]: ./struct.BasicHeader.html
/// [`MessageHeader`]: ./enum.MessageHeader.html
/// [ChunkData]: ./enum.ChunkData.html
#[derive(Debug, Clone)]
pub struct Chunk {
    basic_header: BasicHeader,
    extended_timestamp: Option<Duration>,
    message_header: MessageHeader,
    chunk_data: Option<ChunkData>
}

impl Chunk {
    /// Constructs a new Chunk.
    ///
    /// # Parameters
    ///
    /// * `basic_header: BasicHeader`
    ///
    /// The struct of the chunk basic header.
    ///
    /// * `extended_timestamp: Option<Duration>`
    ///
    /// The timestamp as the `Duration`.
    /// This is efficient than to use either `SystemTime` or to use `Instant` because they will need the `Duration` for time calculation.
    /// If the timestamp of this chunk hasn't been extended, this should be input `None`.
    ///
    /// * `message_header: MessageHeader`
    ///
    /// The struct of the chunk message header.
    ///
    /// * `chunk_data: Option<ChunkData>`
    ///
    /// The struct of the chunk data.
    /// The chunk data will be nothing if the message length is 0.
    /// In that case, this should be input `None`.
    pub fn new(basic_header: BasicHeader, extended_timestamp: Option<Duration>, message_header: MessageHeader, chunk_data: Option<ChunkData>) -> Self {
        Chunk {
            basic_header,
            extended_timestamp,
            message_header,
            chunk_data
        }
    }

    /// Returns the chunk basic header.
    pub fn get_basic_header(&self) -> BasicHeader {
        self.basic_header
    }

    /// Returns the extended timestamp.
    /// This will return the `None` if the timestamp isn't extended.
    pub fn get_extended_timestamp(&self) -> Option<Duration> {
        self.extended_timestamp
    }

    /// Returns the chunk message header.
    pub fn get_message_header(&self) -> MessageHeader {
        self.message_header
    }

    /// Returns the chunk data.
    /// This will return the `&None` if the message length of the chunk data is 0.
    pub fn get_chunk_data(&self) -> &Option<ChunkData> {
        &self.chunk_data
    }
}

/// # The byte buffer to encode/decode the Rust's data/the bytes
///
/// This consists of following data:
///
/// |Field |Type     |
/// | :--- | :------ |
/// |offset|`usize`  |
/// |len   |`usize`  |
/// |bytes |`Vec<u8>`|
///
/// This is implemented to premise that will dispose after receiving/sending the chunk.
/// Therefore this is considered just to append the bytes, not to overwrite its bytes.
#[derive(Debug)]
pub struct ByteBuffer {
    offset: usize,
    len: usize,
    bytes: Vec<u8>
}

impl ByteBuffer {
    /// Constructs a new `ByteBuffer`.
    ///
    /// # Parameters
    ///
    /// * `bytes: Vec<u8>`
    ///
    /// The bytes to decode/encode.
    /// When we start to encode, pass to an empty Vec.
    pub fn new(bytes: Vec<u8>) -> Self {
        ByteBuffer {
            offset: 0,
            len: bytes.len(),
            bytes
        }
    }

    /// Clear this byte buffer.
    /// This sets 0 to the offset, sets 0 to the len, and constructs a new Vec to the bytes again.
    pub fn clear(&mut self) {
        self.offset = 0;
        self.len = 0;
        self.bytes = Vec::new();
    }

    /// Returns the offset.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Forwards the offset.
    ///
    /// # Parameters
    ///
    /// * `offset: usize`
    ///
    /// The offset to forward.
    ///
    /// Note that we should increase the offset when have decoded some byte.
    pub fn offset_to(&mut self, offset: usize) {
        self.offset += offset;
    }

    /// Returns the length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Adds the length.
    ///
    /// # Parameters
    ///
    /// * `len: usize`
    ///
    /// The length to add.
    ///
    /// Note that we should increase the length when have encoded some byte.
    pub fn add_len(&mut self, len: usize) {
        self.len += len;
    }

    /// Returns all byte.
    pub fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    /// Returns all byte as mutable.
    pub fn bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }
}
