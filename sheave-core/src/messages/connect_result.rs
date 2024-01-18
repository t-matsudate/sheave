use std::io::Result as IOResult;
use super::{
    Channel,
    ChunkData,
    Command,
    ensure_command_name,
    headers::MessageType
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer,
    messages::amf::v0::{
        Number,
        AmfString,
        Object
    }
};

/// The response message for connect requests.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectResult {
    result: AmfString,
    properties: Object,
    information: Object
}

impl ConnectResult {
    const TRANSACTION_ID: f64 = 1f64;

    /// Constructs a connect result message.
    pub fn new(result: AmfString, properties: Object, information: Object) -> Self {
        Self { result, properties, information }
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

impl Default for ConnectResult {
    fn default() -> Self {
        Self {
            result: "_result".into(),
            properties: Object::default(),
            information: Object::default()
        }
    }
}

impl ChunkData for ConnectResult {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for ConnectResult {
    /// Gets the result which is either `"_result"` or `"_error"`.
    fn get_command_name(&self) -> &str {
        &**self.result
    }

    /// Gets the transaction ID in this response.
    /// In this response, it's fixed to `1`.
    fn get_transaction_id(&self) -> Number {
        Number::new(Self::TRANSACTION_ID)
    }
}

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
    /// * [`InvalidString`]
    ///
    /// When some value is invalid for UTF-8 string.
    ///
    /// * [`InconsistentCommand`]
    ///
    /// When the command name is neither `"_result"` nor `"_error"`.
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
    ///         amf::v0::{
    ///             Marker,
    ///             Number,
    ///             AmfString,
    ///             Object
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("_result"));
    /// buffer.encode(&Number::new(1f64));
    /// buffer.encode(&Object::default());
    /// buffer.encode(&Object::default());
    /// assert!(Decoder::<ConnectResult>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// buffer.encode(&Number::new(1f64));
    /// buffer.encode(&Object::default());
    /// buffer.encode(&Object::default());
    /// assert!(Decoder::<ConnectResult>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    fn decode(&mut self) -> IOResult<ConnectResult> {
        let result: AmfString = self.decode()?;
        ensure_command_name("_result", result.clone()).or(ensure_command_name("_error", result.clone()))?;
        Decoder::<Number>::decode(self)?;
        let properties: Object = self.decode()?;
        let information: Object = self.decode()?;
        Ok(ConnectResult { result, properties, information } )
    }
}

impl Encoder<ConnectResult> for ByteBuffer {
    /// Encodes a ConnectResult command into bytes.
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
    ///         amf::v0::{
    ///             Number,
    ///             AmfString,
    ///             Object
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&ConnectResult::default());
    /// let command_name: AmfString = buffer.decode().unwrap();
    /// let transaction_id: Number = buffer.decode().unwrap();
    /// let properties: Object = buffer.decode().unwrap();
    /// let information: Object = buffer.decode().unwrap();
    /// assert_eq!("_result", command_name);
    /// assert_eq!(1f64, transaction_id);
    /// assert_eq!(Object::default(), properties);
    /// assert_eq!(Object::default(), information)
    /// ```
    fn encode(&mut self, connect_result: &ConnectResult) {
        self.encode(&AmfString::from(connect_result.get_command_name()));
        self.encode(&connect_result.get_transaction_id());
        self.encode(connect_result.get_properties());
        self.encode(connect_result.get_information());
    }
}

#[cfg(test)]
mod tests {
    use crate::object;
    use super::*;

    #[test]
    fn decode_connect_result() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("_result"));
        buffer.encode(&Number::new(1f64));
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
            "_result".into(),
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
}
