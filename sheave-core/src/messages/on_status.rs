mod publishing_failure;

use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::{
        Channel,
        ChunkData,
        Command,
        amf::v0::{
            Object,
            Null
        },
        headers::MessageType
    }
};
pub use self::publishing_failure::*;

/// The response message for Publish requests.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OnStatus(Object);

impl OnStatus {
    /// Constructs an OnStatus command.
    pub fn new(info_object: Object) -> Self {
        Self(info_object)
    }

    /// Gets the info object.
    pub fn get_info_object(&self) -> &Object {
        &self.0
    }
}

impl From<OnStatus> for Object {
    fn from(on_status: OnStatus) -> Self {
        on_status.0
    }
}

impl ChunkData for OnStatus {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for OnStatus {}

impl Decoder<OnStatus> for ByteBuffer {
    /// Decodes bytes into an OnStatus command.
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
    ///         OnStatus,
    ///         amf::v0::{
    ///             Object,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// buffer.encode(&Object::default());
    /// assert!(Decoder::<OnStatus>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<OnStatus>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<OnStatus> {
        Decoder::<Null>::decode(self)?;
        let info_object: Object = self.decode()?;

        Ok(OnStatus(info_object))
    }
}

impl Encoder<OnStatus> for ByteBuffer {
    /// Encodes an OnStatus command into bytes.
    fn encode(&mut self, on_status: &OnStatus) {
        self.encode(&Null);
        self.encode(on_status.get_info_object());
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
    fn decode_on_status() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        buffer.encode(
            &object!(
                "level" => AmfString::from("status"),
                "code" => AmfString::from("NetStream.Publish.Start"),
                "description" => AmfString::new(format!("{} is now published", "filename")),
                "details" => AmfString::from("filename")
            )
        );
        let result: IOResult<OnStatus> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = OnStatus::new(
            object!(
                "level" => AmfString::from("status"),
                "code" => AmfString::from("NetStream.Publish.Start"),
                "description" => AmfString::new(format!("{} is now published", "filename")),
                "details" => AmfString::from("filename")
            )
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn encode_on_status() {
        let mut buffer = ByteBuffer::default();
        let expected_info_object = object!(
            "level" => AmfString::from("status"),
            "code" => AmfString::from("NetStream.Publish.Start"),
            "description" => AmfString::new(format!("{} is now published", "filename")),
            "details" => AmfString::from("filename")
        );
        buffer.encode(&OnStatus::new(expected_info_object.clone()));
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_info_object: Object = buffer.decode().unwrap();
        assert_eq!(expected_info_object, actual_info_object)
    }
}
