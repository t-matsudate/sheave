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
#[derive(Debug, Clone, PartialEq)]
pub struct Play {
    stream_name: AmfString,
    start_time: Duration,
    play_mode: PlayMode
}

impl Play {
    /// Constructs a Play command.
    pub fn new(stream_name: AmfString, start_time: Duration, play_mode: PlayMode) -> Self {
        Self {
            stream_name,
            start_time,
            play_mode
        }
    }

    /// Gets the stream name. (e.g. filename)
    pub fn get_stream_name(&self) -> &AmfString {
        &self.stream_name
    }

    /// Gets the start time.
    pub fn get_start_time(&self) -> Duration {
        self.start_time
    }

    /// Gets the play mode.
    pub fn get_play_mode(&self) -> PlayMode {
        self.play_mode
    }
}

impl From<Play> for (AmfString, Duration, PlayMode) {
    fn from(play: Play) -> Self {
        (play.stream_name, play.start_time, play.play_mode)
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
        let play_mode: PlayMode = (start_time / 1000f64).into();

        let start_time = if start_time < 0f64 {
            Duration::default()
        } else {
            Duration::from_secs(start_time.as_integer())
        };
        Ok(Play { stream_name, start_time, play_mode })
    }
}

impl Encoder<Play> for ByteBuffer {
    /// Encodes a Play command into bytes.
    fn encode(&mut self, play: &Play) {
        use PlayMode::*;

        let start_time = match play.get_play_mode() {
            Both => -2000f64,
            Live => -1000f64,
            Recorded => (play.get_start_time().as_secs() * 1000) as f64,
            Other => unimplemented!("Play mode is neither both, live nor recorded.")
        };

        self.encode(&Null);
        self.encode(play.get_stream_name());
        self.encode(&Number::new(start_time));
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
