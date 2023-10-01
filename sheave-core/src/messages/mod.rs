//! # The RTMP Messages
//!
//! After handshake, both are required to negosiate what are needed for sending/receiving.
//! It consists of following steps:
//!
//! 1. Connect
//!
//! ## Connect
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
//! Every chunk has the chunk headers.
//! See [`sheave_core::messages::headers`] about them.
//!
//! [the Action Message Format (v0)]: amf::v0
//! [`sheave_core::messages::headers`]: headers

pub mod headers;
pub mod amf;
mod inconsistent_command;
mod connect;

use std::io::Result as IOResult;
use self::amf::v0::AmfString;
pub use self::inconsistent_command::*;
pub use self::connect::*;

#[doc(hidden)]
pub(self) fn ensure_command_name(expected: &str, actual: AmfString) -> IOResult<()> {
    (expected == actual).then_some(()).ok_or(inconsistent_command(expected, actual))
}
