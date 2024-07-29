use std::io::Result as IOResult;
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
    messages::amf::v0::Object,
};

/// The response message for Connect requests.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConnectResult {
    properties: Object,
    information: Object
}

impl ConnectResult {
    /// Constructs a ConnectResult command.
    pub fn new(properties: Object, information: Object) -> Self {
        Self { properties, information }
    }

    /// Gets the properties object.
    pub fn get_properties(&self) -> &Object {
        &self.properties
    }

    /// Gets the information object.
    pub fn get_information(&self) -> &Object {
        &self.information
    }
}

impl From<ConnectResult> for (Object, Object) {
    fn from(connect_result: ConnectResult) -> Self {
        (connect_result.properties, connect_result.information)
    }
}

impl ChunkData for ConnectResult {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for ConnectResult {}

impl Decoder<ConnectResult> for ByteBuffer {
    /// Decodes bytes into a ConnectResult command.
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
    ///         ConnectResult,
    ///         amf::v0::Object,
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Object::default());
    /// buffer.encode(&Object::default());
    /// assert!(Decoder::<ConnectResult>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<ConnectResult>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<ConnectResult> {
        let properties: Object = self.decode()?;
        let information: Object = self.decode()?;
        Ok(ConnectResult { properties, information } )
    }
}

impl Encoder<ConnectResult> for ByteBuffer {
    /// Encodes a ConnectResult command into bytes.
    fn encode(&mut self, connect_result: &ConnectResult) {
        self.encode(connect_result.get_properties());
        self.encode(connect_result.get_information());
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::amf::v0::{
            Number,
            AmfString
        },
        object
    };
    use super::*;

    #[test]
    fn decode_connect_result() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &object!(
                "fmsVer" => AmfString::from("FMS/5,0,17"),
                "capabilities" => Number::new(31f64)
            )
        );
        buffer.encode(
            &object!(
                "level" => AmfString::from("status"),
                "code" => AmfString::from("NetConnection.Connect.Success"),
                "description" => AmfString::from("Connection succeeded."),
                "objectEncoding" => Number::new(0f64)
            )
        );
        let result: IOResult<ConnectResult> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = ConnectResult::new(
            object!(
                "fmsVer" => AmfString::from("FMS/5,0,17"),
                "capabilities" => Number::new(31f64)
            ),
            object!(
                "level" => AmfString::from("status"),
                "code" => AmfString::from("NetConnection.Connect.Success"),
                "description" => AmfString::from("Connection succeeded."),
                "objectEncoding" => Number::new(0f64)
            )
        );
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_connect_result() {
        let mut buffer = ByteBuffer::default();
        let expected_properties = object!(
            "fmsVer" => AmfString::from("FMS/5,0,17"),
            "capabilities" => Number::new(31f64)
        );
        let expected_information = object!(
            "level" => AmfString::from("status"),
            "code" => AmfString::from("NetConnection.Connect.Success"),
            "description" => AmfString::from("Connection succeeded."),
            "objectEncoding" => Number::new(0f64)
        );
        buffer.encode(&ConnectResult::new(expected_properties.clone(), expected_information.clone()));
        let actual_properties: Object = buffer.decode().unwrap();
        assert_eq!(expected_properties, actual_properties);
        let actual_information: Object = buffer.decode().unwrap();
        assert_eq!(expected_information, actual_information)
    }
}
