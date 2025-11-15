mod play_mode;

use std::{
    io::Result as IOResult,
    time::Duration
};
use super::{
    Channel,
    ChunkData,
    Command,
    headers::MessageType
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer,
    messages::amf::v0::{
        AmfString,
        Null,
        Number
    }
};
pub use self::play_mode::*;

/// The command to tell playing information.
///
/// Following format is required:
///
/// |Field|AMF Type|Value|
/// | :- | :- | :- |
/// ||[`Null`]|Nothing but an AMF's type marker is in.|
/// |Stream Name|[`String`]|A name for subscribing a data from the server.|
/// |Start Time|[`Number`]|Time offset of data subscribing.<br />*Note this can be several negative number* (See [Play Mode](#play-mode)).|
///
/// # Play Mode
///
/// |Value|Play Mode|Description|
/// | :- | :- | :- |
/// |`-2`|Both|Subscribes recorded data if its data isn't on a livestream (default).|
/// |`-1`|Live|Subscribes only as livestream.|
/// |`0`<br />(And above)|Recorded|Subscribes only as recorded data.|
///
/// Note the server can treat as some error if its data doesn't exist as specified mode.
///
/// [`Number`]: crate::messages::amf::v0::Number
/// [`String`]: crate::messages::amf::v0::AmfString
/// [`Null`]: crate::messages::amf::v0::Null
#[derive(Debug, Clone, PartialEq)]
pub struct Play {
    stream_name: AmfString,
    start_time: Number
}

impl Play {
    /// Constructs a Play command.
    pub fn new(stream_name: AmfString, start_time: Number) -> Self {
        Self {
            stream_name,
            start_time
        }
    }

    /// Gets the stream name. (e.g. filename)
    pub fn get_stream_name(&self) -> &AmfString {
        &self.stream_name
    }

    /// Gets the start time.
    pub fn get_start_time(&self) -> Number {
        self.start_time
    }

    /// Gets the play mode.
    pub fn get_play_mode(&self) -> PlayMode {
        self.start_time.into()
    }
}

impl From<Play> for (AmfString, Option<Duration>, PlayMode) {
    fn from(play: Play) -> Self {
        let start_time = play.start_time;
        let start_time = if start_time >= 0f64 {
            Some(Duration::from_millis(start_time.as_integer()))
        } else {
            None
        };
        let play_mode = play.get_play_mode();

        (play.stream_name, start_time, play_mode)
    }
}

impl ChunkData for Play {
    const CHANNEL: Channel = Channel::Source;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for Play {}

impl Decoder<Play> for ByteBuffer {
    /// Decodes bytes into a Play command.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When some field misses.
    ///
    /// * [`InconsistentMarker`]
    ///
    /// When some value is inconsistent with its marker.
    ///
    /// * [`InvalidString`]
    ///
    /// When some value is invalid for UTF-8 string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         Play,
    ///         amf::v0::{
    ///             AmfString,
    ///             Null,
    ///             Number
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// buffer.encode(&Number::new(-2000f64));
    /// assert!(Decoder::<Play>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<Play>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    fn decode(&mut self) -> IOResult<Play> {
        Decoder::<Null>::decode(self)?;
        let stream_name: AmfString = self.decode()?;
        let start_time: Number = self.decode()?;

        Ok(Play { stream_name, start_time })
    }
}

impl Encoder<Play> for ByteBuffer {
    /// Encodes a Play command into bytes.
    fn encode(&mut self, play: &Play) {
        self.encode(&Null);
        self.encode(play.get_stream_name());
        self.encode(&play.get_start_time());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_play() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());
        buffer.encode(&Number::new(-2000f64));
        let result: IOResult<Play> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = Play::new(AmfString::default(), Number::new(-2000f64));
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_play() {
        let mut buffer = ByteBuffer::default();
        let expected_stream_name = "";
        let expected_start_time = -2000f64;
        let expected = Play::new(AmfString::from(expected_stream_name), Number::new(expected_start_time));
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_stream_name: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_stream_name, actual_stream_name);
        let actual_start_time: Number = buffer.decode().unwrap();
        assert_eq!(expected_start_time, actual_start_time)
    }
}
