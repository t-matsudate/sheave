use std::io::{
    Error as IOError,
    Result as IOResult
};
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::{
        SetDataFrame,
        amf::v0::{
            AmfString,
            EcmaArray
        }
    }
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptDataTag {
    name: AmfString,
    value: EcmaArray
}

impl ScriptDataTag {
    pub fn new(name: AmfString, value: EcmaArray) -> Self {
        Self { name, value }
    }

    pub fn get_name(&self) -> &AmfString {
        &self.name
    }

    pub fn get_value(&self) -> &EcmaArray {
        &self.value
    }
}

impl Decoder<ScriptDataTag> for ByteBuffer {
    /// Decodes a SetDataFrame message into a ScriptDataTag.
    ///
    /// # Erorrs
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
    ///     flv::ScriptDataTag,
    ///     messages::amf::v0::{
    ///         AmfString,
    ///         EcmaArray
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("onMetaData"));
    /// buffer.encode(&EcmaArray::default());
    /// assert!(Decoder::<ScriptDataTag>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<ScriptDataTag>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    fn decode(&mut self) -> IOResult<ScriptDataTag> {
        let name: AmfString = self.decode()?;
        let value: EcmaArray = self.decode()?;
        Ok(ScriptDataTag { name, value })
    }
}

impl Encoder<ScriptDataTag> for ByteBuffer {
    /// Encodes a ScriptDataTag into a SetDataFrame message.
    fn encode(&mut self, script_data: &ScriptDataTag) {
        self.encode(script_data.get_name());
        self.encode(script_data.get_value());
    }
}

impl TryFrom<SetDataFrame> for ScriptDataTag {
    type Error = IOError;

    fn try_from(set_data_frame: SetDataFrame) -> IOResult<Self> {
        let mut buffer: ByteBuffer = Vec::<u8>::from(set_data_frame).into();
        Decoder::<Self>::decode(&mut buffer)
    }
}

impl TryFrom<ScriptDataTag> for SetDataFrame {
    type Error = IOError;

    fn try_from(script_data_tag: ScriptDataTag) -> IOResult<Self> {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&script_data_tag);
        Ok(Self::new(buffer.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ecma_array,
        messages::amf::v0::{
            Number,
            Boolean
        }
    };
    use super::*;

    #[test]
    fn decode_script_data() {
        let mut buffer = ByteBuffer::default();
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
        let result: IOResult<ScriptDataTag> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = ScriptDataTag::new(
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
    fn encode_script_data() {
        let mut buffer = ByteBuffer::default();
        let expected_name = "onMetaData";
        let expected_value = ecma_array!(
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
        let expected = ScriptDataTag::new(expected_name.into(), expected_value.clone());
        buffer.encode(&expected);

        let actual_name: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_name, actual_name);
        let actual_value: EcmaArray = buffer.decode().unwrap();
        assert_eq!(expected_value, actual_value)
    }
}
