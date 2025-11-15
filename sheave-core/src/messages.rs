//! # The RTMP Messages
//!
//! After handshake, both are required to negotiate what are needed for sending/receiving.
//! Currently, following messages are used in negotiation:
//! 
//! * User Control
//! * [`Window Acknowledgement Size`]
//! * Command
//!
//! Every message has headers.
//! See [`sheave_core::messages::headers`] about them.
//!
//! ## User Control
//!
//! User Control messages are additional informations for starting streaming.
//! These consist of following format:
//!
//! |Field|Length (in bytes)|Description|
//! | :- | -: | :- |
//! |Event Type|2|A marker which indicates a kind of data sent.|
//! |Event Data|Various|Actual data of this message.|
//!
//! ### Events
//!
//! Currently, following events are used:
//!
//! * [`Stream Begin`]
//! * [`Set Buffer Length`]
//!
//! ## Command
//!
//! Command messages are contained basic informations actually required to negotiate.
//! These encodes/decodes into/from [`the Action Message Format 0`].
//! These consist of following format:
//!
//! |Field|AMF Type|Description|
//! | :- | :- | :- |
//! |Command Name|[`String`]|The command which is currently negotiated.|
//! |Transaction ID|[`Number`]|A number which indicates a step of this negotiation.|
//! |Setting Data|Various|Actually negotiated data.|
//!
//! Furthermore commands have following patterns:
//!
//! * NetConnection
//! * NetStream
//!
//! ### NetConnection
//!
//! NetConnection commands are defined as following messages:
//!
//! * [`connect`]
//! * [`releaseStream`]
//! * [`FCPublish`]
//! * [`createStream`]
//! * [`FCSubscribe`]
//! * [`getStreamLength`] (in FFmpeg) / [`set_playlist`] (in OBS) (note these mayn't be sent.)
//!
//! ### NetStream
//!
//! NetStream commands are defined as following messages:
//!
//! * [`publish`]
//! * [`play`]
//!
//! ## Any data
//!
//! After negotiation, the server receives actual audio/video data from clients.
//! However also FLV metadata is received as the AMF (v0).
//! The message what contains FLV metadata is called [`@setDataFrame`].
//! This consists of following format:
//!
//! |Field|AMF Type|Description|
//! | :- | :- | :- |
//! |Command Name (probably)|[`String`]|`"@setDataFrame"`|
//! |Setting Data|Various|In this case, see [`ScriptDataTag`].|
//!
//! [the Action Message Format 0]: amf::v0
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
//! [`@setDataFrame`]: SetDataFrame
//! [`ScriptDataTag`]: crate::flv::tags::ScriptDataTag
//! [`Window Acknowledgement Size`]: WindowAcknowledgementSize
//! [`FCSubscribe`]: FcSubscribe
//! [`getStreamLength`]: GetStreamLength
//! [`set_playlist`]: SetPlaylist
//! [`play`]: Play
//! [`Set Buffer Length`]: SetBufferLength

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
mod window_acknowledgement_size;
mod peer_bandwidth;
mod fc_unpublish;
mod delete_stream;
mod fc_subscribe;
mod get_stream_length;
mod get_stream_length_result;
mod set_playlist;
mod playlist_ready;
mod play;
mod set_buffer_length;
mod command_error;

use std::cmp::Ordering;
use self::headers::MessageType;
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
    acknowledgement::*,
    window_acknowledgement_size::*,
    peer_bandwidth::*,
    fc_unpublish::*,
    delete_stream::*,
    fc_subscribe::*,
    get_stream_length::*,
    get_stream_length_result::*,
    set_playlist::*,
    playlist_ready::*,
    play::*,
    set_buffer_length::*,
    command_error::*
};

/// The IDs which are assigned every roles of chunks.
/// This is mainly used for the [`BasicHeader`]'s chunk ID.
///
/// [`BasicHeader`]: headers::BasicHeader
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
/// When writing into streams, we are required to imprint a chunk ID and a message type ID into their headers.
/// This makes you to reduce its cost.
/// For example, [`write_chunk`] use this for writing chunks correctly.
///
/// [`write_chunk`]: crate::writers::write_chunk
pub trait ChunkData {
    const CHANNEL: Channel;
    const MESSAGE_TYPE: MessageType;
}

pub trait Command {}

/// The IDs which are types of user control messages.
///
/// Variants correspond to respectively following events:
///
/// |Patttern|Event Type|
/// | :- | :- |
/// |`StreamBegin`|[`StreamBegin`]|
/// |`SetBufferLength`|[`SetBufferLength`]|
/// |`Other`|other event type|
///
/// [`StreamBegin`]: StreamBegin
/// [`SetBufferLength`]: SetBufferLength
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    StreamBegin,
    SetBufferLength = 3,
    Other = 0xffff
}

impl From<u16> for EventType {
    fn from(event_type: u16) -> Self {
        use EventType::*;

        match event_type {
            0 => StreamBegin,
            3 => SetBufferLength,
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

impl PartialEq<WindowAcknowledgementSize> for Acknowledgement {
    fn eq(&self, other: &WindowAcknowledgementSize) -> bool {
        self.get_inner().eq(&other.get_inner())
    }
}

impl PartialOrd<WindowAcknowledgementSize> for Acknowledgement {
    fn partial_cmp(&self, other: &WindowAcknowledgementSize) -> Option<Ordering> {
        self.get_inner().partial_cmp(&other.get_inner())
    }
}

impl PartialEq<Acknowledgement> for WindowAcknowledgementSize {
    fn eq(&self, other: &Acknowledgement) -> bool {
        self.get_inner().eq(&other.get_inner())
    }
}

impl PartialOrd<Acknowledgement> for WindowAcknowledgementSize {
    fn partial_cmp(&self, other: &Acknowledgement) -> Option<Ordering> {
        self.get_inner().partial_cmp(&other.get_inner())
    }
}

impl PartialEq<PeerBandwidth> for Acknowledgement {
    fn eq(&self, other: &PeerBandwidth) -> bool {
        self.get_inner().eq(&other.get_inner_bandwidth())
    }
}

impl PartialOrd<PeerBandwidth> for Acknowledgement {
    fn partial_cmp(&self, other: &PeerBandwidth) -> Option<Ordering> {
        self.get_inner().partial_cmp(&other.get_inner_bandwidth())
    }
}

impl PartialEq<Acknowledgement> for PeerBandwidth {
    fn eq(&self, other: &Acknowledgement) -> bool {
        self.get_inner_bandwidth().eq(&other.get_inner())
    }
}

impl PartialOrd<Acknowledgement> for PeerBandwidth {
    fn partial_cmp(&self, other: &Acknowledgement) -> Option<Ordering> {
        self.get_inner_bandwidth().partial_cmp(&other.get_inner())
    }
}
