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
    messages::amf::v0::{
        Number,
        Object
    }
};

/// The command to tell the information that where connects from/to.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Connect(Object);

impl Connect {
    const TRANSACTION_ID: f64 = 1f64;

    /// Constructs a Connect command.
    pub fn new(command_object: Object) -> Self {
        Self(command_object)
    }

    /// Gets the command object in this request.
    pub fn get_command_object(&self) -> &Object {
        &self.0
    }
}

impl From<Connect> for Object {
    fn from(connect: Connect) -> Self {
        connect.0
    }
}

impl Decoder<Connect> for ByteBuffer {
    /// Decodes bytes into a Connect command.
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
    ///         Connect,
    ///         amf::v0::{
    ///             Number,
    ///             Object
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Number::new(1f64));
    /// buffer.encode(&Object::default());
    /// assert!(Decoder::<Connect>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<Connect>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<Connect> {
        // Skips because the transaction ID in the Connect command is fixed to `1`.
        Decoder::<Number>::decode(self)?;
        let command_object: Object = self.decode()?;
        Ok(Connect(command_object))
    }
}

impl Encoder<Connect> for ByteBuffer {
    /// Encodes a Connect command into bytes.
    fn encode(&mut self, connect: &Connect) {
        self.encode(&connect.get_transaction_id());
        self.encode(connect.get_command_object());
    }
}

impl ChunkData for Connect {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for Connect {
    /// Gets the transaction ID.
    /// In this request, it's fixed to `1`.
    fn get_transaction_id(&self) -> Number {
        Number::new(Self::TRANSACTION_ID)
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
    fn decode_connect_input() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Number::new(1f64));
        buffer.encode(
            &object!(
                "app" => AmfString::from("ondemand"),
                "type" => AmfString::from("nonprivate"),
                "flashVer" => AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"),
                "tcUrl" => AmfString::from("rtmp://localhost")
            )
        );
        let result: IOResult<Connect> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = Connect::new(
            object!(
                "app" => AmfString::from("ondemand"),
                "type" => AmfString::from("nonprivate"),
                "flashVer" => AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"),
                "tcUrl" => AmfString::from("rtmp://localhost")
            )
        );
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_connect_input() {
        let mut buffer = ByteBuffer::default();
        let expected_transaction_id = 1f64;
        let expected_command_object = object!(
            "app" => AmfString::from("ondemand"),
            "type" => AmfString::from("nonprivate"),
            "flashVer" => AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"),
            "tcUrl" => AmfString::from("rtmp://localhost")
        );
        let expected = Connect::new(expected_command_object.clone());
        buffer.encode(&expected);
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id);
        let actual_command_object: Object = buffer.decode().unwrap();
        assert_eq!(expected_command_object, actual_command_object)
    }
}
