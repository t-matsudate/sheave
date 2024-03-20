//! # The RTMP Messages
//!
//! After handshake, both are required to negosiate what are needed for sending/receiving.
//! It consists of following steps:
//!
//! 1. [`connect`]
//! 2. [`releaseStream`]
//! 3. [`FCPublish`]
//! 4. [`createStream`]
//! 5. [`publish`]
//!
//! ## [`connect`]
//!
//! Exchanges informations about their applications each other.
//! In this case, next format is required:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|`String`|`"connect"`|
//! |Transaction ID|`Number`|`1`|
//! |Command Object|`Object`|See [Command Object](#command-object).|
//!
//! In the command message, we negotiate by using [the Action Message Format (v0)].
//!
//! <h3><a id="command-object">Command Object</a></h3>
//!
//! The Command Object is written detailed informations of both applications.
//! Concretely, several of following pair is exchanged:
//!
//! |Key|AMF Type|Description|
//! | :- | :- | :- |
//! |app|[`String`]|Server application name.|
//! |flashVer|[`String`]|Either a flash player version(client) or an FLV encoder version(server).|
//! |swfUrl|[`String`]|The source SWF file URL.|
//! |tcURL|[`String`]|The URL to connect server as the TCP. `app` value is included in this value.|
//! |fpad|[`Boolean`]|Whether a proxy is used.|
//! |audioCodecs|[`Number`]|Supported audio codecs.|
//! |videoCodecs|[`Number`]|Supported video codecs.|
//! |videoFunctions|[`Number`]|Supported video functions.|
//! |pageUrl|[`String`]|The URL of web page which SWF file is loaded.|
//! |objectEncoding|[`Number`]|A version of the Action Message Format.|
//!
//! Note: Something not in above can be exchanged.
//!
//! Against, the result for connect request is required to be next format:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|[`String`]|`"_result"` or `"_error"`|
//! |Transaction ID|[`Number`]|Same as the connect request|
//! |Properties|[`Object`]|e.g. server-side informations.|
//! |Information|[`Object`]|Contents for describing its response.|
//!
//! ## [`releaseStream`]
//!
//! Tells "Play Path" to the parter.
//! In this case, next format is required:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|[`String`]|`"releaseStream"`|
//! |Transaction ID|[`Number`]|A number which is next of the connect.|
//! ||[`Null`]|Nothing but an AMF's type marker is in.|
//! |Play Path|[`String`]|Any string is in if it is specified.|
//!
//! For example, Play Path is defined as some file name in [FFmpeg](https://github.com/FFmpeg/FFmpeg/blob/master/libavformat/rtmpproto.c#L2624-L2625).
//!
//! Against, the result for releaseStream request is required to be next format:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|[`String`]|`"_result"` or `"_error"`|
//! |Transaction ID|[`Number`]|Same as the releaseStream request.|
//! ||[`Null`]|Nothing but an AMF's type marker is in.|
//!
//! ## [`FCPublish`]
//!
//! Tells to the partner what is same as the releaseStream request.
//! In this case, next format is required:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|[`String`]|`"FCPublish"`|
//! |Transaction ID|[`Number`]|A number which is next of the releaseStream.|
//! ||[`Null`]|Nothing but an AMF's type marker is in.|
//! |Play Path|[`String`]|(Probably) Same as the releaseStream request.|
//!
//! Against, the result for FCPublish request is required to be next format:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|[`String`]|`"onFCPublish"`|
//!
//! ## [`createStream`]
//!
//! Requests to create a new Message Stream ID to the partner.
//! In this case, next format is required:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|[`String`]|`"createStream"`|
//! |Transaction ID|[`Number`]|A number which is next of the FCPublish.|
//! ||[`Null`]|Nothing but an AMF's type marker is in.|
//!
//! Against, the result for createStream request is required to be next format:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|[`String`]|`"_result"` or `"_error"`|
//! |Transaction ID|[`Number`]|Same as the createStream request.|
//! ||[`Null`]|Nothing but an AMF's type marker is in.|
//! |Message ID|[`Number`]|A Message Stream ID which is assigned by the partner.|
//!
//! ## [`publish`]
//!
//! Tells to the partner what is an way to publish its stream.
//! In this case, next format is required:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|[`String`]|`"publish"`|
//! |Transaction ID|[`Number`]|A number which is next of the createStream.|
//! ||[`Null`]|Nothing but an AMF's type marker is in.|
//! |Publishing Name|[`String`]|A name for publishing its stream to the partner.|
//! |Publishing Type|[`String`]|Either `"live"`, `"record"` or `"append"`. See [Publishing Type](#publishing-type).|
//!
//! Against, result**s** for publish request **are** required to be next format:
//!
//! 1. [`Stream Begin`]
//! 2. [`onStatus`]
//!
//! ### [`Stream Begin`]
//!
//! This is an event message of the User Control Message.
//! This has following data:
//!
//! 1. Event Type (2 bytes / `0`)
//! 2. A Message ID (4 bytes.)
//!
//! ### [`onStatus`]
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|[`String`]|`"onStatus"`|
//! |Transaction ID|[`Number`]|`0`|
//! ||[`Null`]|Nothing but an AMF's type marker is in.|
//! |Info Object|[`Object`]|Similar to the information of connect result.|
//!
//! <h3><a id="publishing-type">Publishing Type</a></h3>
//!
//! The publish command requires you to specify one of "Publishing Type" in its request.
//! Publishing Type means:
//!
//! * `"live"`
//!
//! Only streaming. Media data will never be stored.
//!
//! * `"record"`
//!
//! Media data will be stored. If publishing name duplicated, it is rewritten as a new file.
//!
//! * `"append"`
//!
//! Same as `"record"` excepts is appended media data if publishing name duplicated.
//!
//! Every chunk has the chunk headers.
//! See [`sheave_core::messages::headers`] about them.
//!
//! After negotiation, the server receives actual audio/video data from clients.
//! However also FLV metadata is received as the AMF (v0).
//! The message what contains FLV metadata is called [`SetDataFrame`].
//!
//! ## [`SetDataFrame`]
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name (Probably)|[`String`]|`"@setDataFrame"`|
//! |Data Name|[`String`]|`"onMetaData"`|
//! |Data|[`EcmaArray`]|e.g. `"audiocodecid"`, `"videocodecid"`|
//!
//! [the Action Message Format (v0)]: amf::v0
//! [`sheave_core::messages::headers`]: headers
//! [`Number`]: amf::v0::Number
//! [`Boolean`]: amf::v0::Boolean
//! [`String`]: amf::v0::AmfString
//! [`Object`]: amf::v0::Object
//! [`Null`]: amf::v0::Null
//! [`EcmaArray`]: amf::v0::EcmaArray
//! [`connect`]: Connect
//! [`releaseStream`]: ReleaseStream
//! [`FCPublish`]: FcPublish
//! [`createStream`]: CreateStream
//! [`publish`]: Publish
//! [`Stream Begin`]: StreamBegin
//! [`onStatus`]: OnStatus
//! [`SetDataFrame`]: SetDataFrame

pub mod headers;
pub mod amf;
mod inconsistent_command;
mod connect;
mod chunk_size;
mod connect_result;
mod release_stream;
mod release_stream_result;
mod fc_publish;
mod on_fc_publish;
mod create_stream;
mod create_stream_result;
mod publish;
mod inconsistent_event_type;
mod stream_begin;
mod on_status;
mod set_data_frame;
mod audio;
mod video;
mod acknowledgement;

use std::io::Result as IOResult;
use self::{
    amf::v0::{
        AmfString,
        Number,
    },
    headers::MessageType
};
pub use self::{
    inconsistent_command::*,
    connect::*,
    chunk_size::*,
    connect_result::*,
    release_stream::*,
    release_stream_result::*,
    fc_publish::*,
    on_fc_publish::*,
    create_stream::*,
    create_stream_result::*,
    publish::*,
    inconsistent_event_type::*,
    stream_begin::*,
    on_status::*,
    set_data_frame::*,
    audio::*,
    video::*,
    acknowledgement::*
};

#[doc(hidden)]
pub(self) fn ensure_command_name(expected: &str, actual: AmfString) -> IOResult<()> {
    (expected == actual).then_some(()).ok_or(inconsistent_command(expected, actual))
}

#[doc(hidden)]
pub(self) fn ensure_event_type(expected: EventType, actual: u16) -> IOResult<()> {
    (expected == EventType::from(actual)).then_some(()).ok_or(inconsistent_event_type(expected, actual))
}

/// The IDs which are assigned every roles of chunks.
/// This is mainly used for the `BasicHeader`'s chunk ID.
///
/// Variants correspond to respectively following chunks:
///
/// |Patttern|Message Type|
/// | :- | :- |
/// |`Network`|[`ChunkSize`]|
/// |`System`|[`Command`]|
/// |`Audio`|[`SetDataFrame`], [`Audio`]|
/// |`Video`|[`Video`]|
/// |`Other`|other chunks|
///
/// [`ChunkSize`]: ChunkSize
/// [`Command`]: Command
/// [`SetDataFrame`]: SetDataFrame
/// [`Audio`]: Audio
/// [`Video`]: Video
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Channel {
    Network = 2,
    System = 3,
    Audio = 4,
    Video = 6,
    Source = 8,
    Other = 0xffff
}

impl From<u16> for Channel {
    fn from(channel: u16) -> Self {
        use Channel::*;

        match channel {
            2 => Network,
            3 => System,
            4 => Audio,
            6 => Video,
            8 => Source,
            _ => Other
        }
    }
}

impl From<Channel> for u16 {
    fn from(channel: Channel) -> u16 {
        channel as u16
    }
}

/// Uniforms chunk data by a channel numbers and a message type.
///
/// When reading from streams or writing into streams, we are required to imprint a chunk ID and a message type ID into their headers.
/// This makes you to reduce its cost.
/// For example, [`read_chunk`] and [`write_chunk`] use this for reading/writing chunks correctly.
///
/// [`read_chunk`]: crate::readers::read_chunk
/// [`write_chunk`]: crate::writers::write_chunk
pub trait ChunkData {
    const CHANNEL: Channel;
    const MESSAGE_TYPE: MessageType;
}

/// The accessor for common fields in every command chunk.
///
/// All command chunk but onFCPublish has its command name and identifier for the transaction (only onFCPublish has just its command name).
/// This provide ways for accessing to their common fields of command chunks uniformly.
/// For example, [`decode`] and [`encode`] which are implemented for them use this for reading them from streams or writing them into streams.
///
/// [`decode`]: crate::Decoder
/// [`encode`]: crate::Encoder
pub trait Command {
    fn get_command_name(&self) -> &str;
    fn get_transaction_id(&self) -> Number;
}

/// The IDs which are types of user control messages.
///
/// Variants correspond to respectively following events:
///
/// |Patttern|Message Type|
/// | :- | :- |
/// |`StreamBegin`|[`StreamBegin`]|
/// |`Other`|other event type|
///
/// [`StreamBegin`]: StreamBegin
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    StreamBegin,
    Other = 0xffff
}

impl From<u16> for EventType {
    fn from(event_type: u16) -> Self {
        use EventType::*;

        match event_type {
            0 => StreamBegin,
            _ => Other
        }
    }
}

impl From<EventType> for u16 {
    fn from(event_type: EventType) -> Self {
        event_type as u16
    }
}

/// Uniforms user control messages by an event type.
///
/// When reading from streams or writing into streams, we are required to imprint a event type ID into their messages.
/// This makes you to reduce its cost.
pub trait UserControl {
    const EVENT_TYPE: EventType;
}
