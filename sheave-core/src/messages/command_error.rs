use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::amf::v0::Object
};
use super::{
    Channel,
    ChunkData,
    Command,
    headers::MessageType
};

/// The response message that some command failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandError(Object);

impl CommandError {
    /// Constructs a CommandError.
    pub fn new(information: Object) -> Self {
        Self(information)
    }

    /// Gets the information object.
    pub fn get_information(&self) -> &Object {
        &self.0
    }
}

impl From<CommandError> for Object {
    fn from(command_error: CommandError) -> Self {
        command_error.0
    }
}

impl ChunkData for CommandError {
    const CHANNEL: Channel = Channel::Source;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for CommandError {}

impl Decoder<CommandError> for ByteBuffer {
    /// Decodes bytes into a CommandError command.
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
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         CommandError,
    ///         amf::v0::Object,
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Object::default());
    /// assert!(Decoder::<CommandError>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<CommandError>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<CommandError> {
        let information: Object = self.decode()?;
        Ok(CommandError(information))
    }
}

impl Encoder<CommandError> for ByteBuffer {
    /// Encodes a CommandError command into bytes.
    fn encode(&mut self, command_error: &CommandError) {
        self.encode(command_error.get_information());
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::amf::v0::AmfString,
        object
    };
    use super::*;

    #[test]
    fn decode_error_input() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetStream.GetStreamLength.MetadataNotFound"),
                "description" => AmfString::from("Metadata field didn't find in specified file.")
            )
        );
        let result: IOResult<CommandError> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = CommandError::new(
            object!(
                "level" => AmfString::from("error"),
                "code" => AmfString::from("NetStream.GetStreamLength.MetadataNotFound"),
                "description" => AmfString::from("Metadata field didn't find in specified file.")
            )
        );
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_error_input() {
        let mut buffer = ByteBuffer::default();
        let expected_information = object!(
            "level" => AmfString::from("error"),
            "code" => AmfString::from("NetStream.GetStreamLength.MetadataNotFound"),
            "description" => AmfString::from("Metadata field didn't find in specified file.")
        );
        let expected = CommandError::new(expected_information.clone());
        buffer.encode(&expected);
        let actual_information: Object = buffer.decode().unwrap();
        assert_eq!(expected_information, actual_information)
    }
}
