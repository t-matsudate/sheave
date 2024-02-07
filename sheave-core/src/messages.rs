//! # The RTMP Messages
//!
//! After handshake, both are required to negosiate what are needed for sending/receiving.
//! It consists of following steps:
//!
//! 1. connect
//! 2. releaseStream
//! 3. FCPublish
//!
//! ## connect
//!
//! Exchanges informations about their applications each other.
//! In this case, next format is required:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|`String`|`"connect"`.|
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
//! |app|`String`|Server application name.|
//! |flashVer|`String`|Either a flash player version(client) or an FLV encoder version(server).|
//! |swfUrl|`String`|The source SWF file URL.|
//! |tcURL|`String`|The URL to connect server as the TCP. `app` value is included in this value.|
//! |fpad|`Boolean`|Whether a proxy is used.|
//! |audioCodecs|`Number`|Supported audio codecs.|
//! |videoCodecs|`Number`|Supported video codecs.|
//! |videoFunctions|`Number`|Supported video functions.|
//! |pageUrl|`String`|The URL of web page which SWF file is loaded.|
//! |objectEncoding|`Number`|A version of the Action Message Format.|
//!
//! Note: Something not in above can be exchanged.
//!
//! Against, the result for connect request is required to be next format:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|`String`|`_result` or `_error`|
//! |Transaction ID|`Number`|Same as the connect request|
//! |Properties|`Object`|e.g. server-side informations.|
//! |Information|`Object`|Contents for describing its response.|
//!
//! ## releaseStream
//!
//! Tells "Play Path" to the parter.
//! In this case, next format is required:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|`String`|`releaseStream`|
//! |Transaction ID|`Number`|A number which is next of the `Connect`.|
//! ||`Null`|Nothing but an AMF's type marker is in.|
//! |Play Path|`String`|Any string is in if it is specified.|
//!
//! For example, Play Path is defined as some file name in [FFmpeg](https://github.com/FFmpeg/FFmpeg/blob/master/libavformat/rtmpproto.c#L2624-L2625).
//!
//! Against, the result for releaseStream request is required to be next format:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|`String`|`_result` or `_error`|
//! |Transaction ID|`Number`|Same as the releaseStream request.|
//! ||`Null`|Nothing but an AMF's type marker is in.|
//!
//! ## FCPublish
//!
//! Tells what is same as the releaseStream request to the partner.
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|`String`|`FCPublish`|
//! |Transaction ID|`Number`|Same as the FCPublish request.|
//! ||`Null`|Nothing but an AMF's type marker is in.|
//! |Play Path|`String`|(Probably) Same as the releaseStream request.|
//!
//! Against, the result for FCPublish request is required to be next format:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |Command Name|`String`|`onFCPublish`|
//!
//! Every chunk has the chunk headers.
//! See [`sheave_core::messages::headers`] about them.
//!
//! [the Action Message Format (v0)]: amf::v0
//! [`sheave_core::messages::headers`]: headers

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
    publish::*
};

#[doc(hidden)]
pub(self) fn ensure_command_name(expected: &str, actual: AmfString) -> IOResult<()> {
    (expected == actual).then_some(()).ok_or(inconsistent_command(expected, actual))
}

/// The IDs which are assigned every roles of chunks.
/// This is mainly used for the `BasicHeader`'s chunk ID.
///
/// Variants correspond to respectively following chunks:
///
/// |Patttern|Message Type|
/// | :- | :- |
/// |`Network`|`ChunkSize`|
/// |`System`|`Command`|
/// |`Other`|other chunks|
///
/// [`ChunkSize`]: ChunkSize
/// [`Command`]: Command
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Channel {
    Network = 2,
    System = 3,
    Source = 8,
    Other = 0xffff
}

impl From<u16> for Channel {
    fn from(channel: u16) -> Self {
        use Channel::*;

        match channel {
            2 => Network,
            3 => System,
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
