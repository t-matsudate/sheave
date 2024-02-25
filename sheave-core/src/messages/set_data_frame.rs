use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::{
        Channel,
        ChunkData,
        amf::v0::{
            AmfString,
            EcmaArray
        },
        ensure_command_name,
        headers::MessageType
    }
};

/// The data message to handle the metadata of FLV.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetDataFrame {
    name: AmfString,
    data: EcmaArray
}

impl SetDataFrame {
    /// Constructs a new SetDataFrame message.
    pub fn new(name: AmfString, data: EcmaArray) -> Self {
        Self { name, data }
    }

    /// Gets the data name of this message.
    pub fn get_name(&self) -> &AmfString {
        &self.name
    }

    /// Gets the data of this message.
    pub fn get_data(&self) -> &EcmaArray {
        &self.data
    }
}

impl From<SetDataFrame> for (AmfString, EcmaArray) {
    fn from(set_data_frame: SetDataFrame) -> Self {
        (set_data_frame.name, set_data_frame.data)
    }
}

impl ChunkData for SetDataFrame {
    const CHANNEL: Channel = Channel::Audio;
    const MESSAGE_TYPE: MessageType = MessageType::Data;
}

impl Decoder<SetDataFrame> for ByteBuffer {
    /// Decodes bytes into a SetDataFrame message.
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
    /// * [`InconsistentCommand`]
    ///
    /// When the command name isn't `"@setDataFrame"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         SetDataFrame,
    ///         amf::v0::{
    ///             Number,
    ///             Boolean,
    ///             AmfString,
    ///             EcmaArray
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("@setDataFrame"));
    /// buffer.encode(&AmfString::from("onMetaData"));
    /// buffer.encode(&EcmaArray::default());
    /// assert!(Decoder::<SetDataFrame>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// buffer.encode(&AmfString::from("onMetaData"));
    /// buffer.encode(&EcmaArray::default());
    /// assert!(Decoder::<SetDataFrame>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    fn decode(&mut self) -> IOResult<SetDataFrame> {
        Decoder::<AmfString>::decode(self).and_then(
            |command| ensure_command_name("@setDataFrame", command)
        )?;

        let name: AmfString = self.decode()?;
        let data: EcmaArray = self.decode()?;
        Ok(SetDataFrame { name, data })
    }
}

impl Encoder<SetDataFrame> for ByteBuffer {
    /// Encodes a SetDataFrame message into bytes.
    fn encode(&mut self, set_data_frame: &SetDataFrame) {
        self.encode(&AmfString::from("@setDataFrame"));
        self.encode(set_data_frame.get_name());
        self.encode(set_data_frame.get_data());
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ecma_array,
        messages::amf::v0::{
            Number,
            Boolean,
            AmfString
        }
    };
    use super::*;

    #[test]
    fn decode_set_data_frame() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("@setDataFrame"));
        buffer.encode(&AmfString::from("onMetaData"));
        buffer.encode(
            &ecma_array!(
                "audiocodecid" => Number::default(),
                "audiodatarate" => Number::default(),
                "audiodelay" => Number::default(),
                "audiosamplerate" => Number::default(),
                "audiosamplesize" => Number::default(),
                "canSeekToEnd" => Boolean::default(),
                "creationdate" => AmfString::default(),
                "duration" => Number::default(),
                "filesize" => Number::default(),
                "framerate" => Number::default(),
                "height" => Number::default(),
                "stereo" => Boolean::default(),
                "videocodecid" => Number::from(2),
                "videodatarate" => Number::default(),
                "width" => Number::default()
            )
        );
        let result: IOResult<SetDataFrame> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = SetDataFrame::new(
            "onMetaData".into(),
            ecma_array!(
                "audiocodecid" => Number::default(),
                "audiodatarate" => Number::default(),
                "audiodelay" => Number::default(),
                "audiosamplerate" => Number::default(),
                "audiosamplesize" => Number::default(),
                "canSeekToEnd" => Boolean::default(),
                "creationdate" => AmfString::default(),
                "duration" => Number::default(),
                "filesize" => Number::default(),
                "framerate" => Number::default(),
                "height" => Number::default(),
                "stereo" => Boolean::default(),
                "videocodecid" => Number::from(2),
                "videodatarate" => Number::default(),
                "width" => Number::default()
            )
        );
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_set_data_frame() {
        let mut buffer = ByteBuffer::default();
        let expected_data_name = "onMetaData";
        let expected_data = ecma_array!(
            "audiocodecid" => Number::default(),
            "audiodatarate" => Number::default(),
            "audiodelay" => Number::default(),
            "audiosamplerate" => Number::default(),
            "audiosamplesize" => Number::default(),
            "canSeekToEnd" => Boolean::default(),
            "creationdate" => AmfString::default(),
            "duration" => Number::default(),
            "filesize" => Number::default(),
            "framerate" => Number::default(),
            "height" => Number::default(),
            "stereo" => Boolean::default(),
            "videocodecid" => Number::from(2),
            "videodatarate" => Number::default(),
            "width" => Number::default()
        );
        let expected = SetDataFrame::new(expected_data_name.into(), expected_data.clone());
        buffer.encode(&expected);
        let message_name: AmfString = buffer.decode().unwrap();
        assert_eq!("@setDataFrame", message_name);
        let actual_data_name: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_data_name, actual_data_name);
        let actual_data: EcmaArray = buffer.decode().unwrap();
        assert_eq!(expected_data, actual_data)
    }
}
