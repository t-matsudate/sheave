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
            Number,
            AmfString,
            Object,
            Null
        },
        ensure_command_name,
        headers::MessageType
    }
};

/// The response message for Publish requests.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OnStatus(Object);

impl OnStatus {
    const COMMAND_NAME: &'static str = "onStatus";
    const TRANSACTION_ID: f64 = 0f64;

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

impl Command for OnStatus {
    fn get_command_name(&self) -> &str {
        Self::COMMAND_NAME
    }

    fn get_transaction_id(&self) -> Number {
        Number::new(Self::TRANSACTION_ID)
    }
}

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
    /// * [`InvalidString`]
    ///
    /// When some value is invalid for UTF-8 string.
    ///
    /// * [`InconsistentCommand`]
    ///
    /// When the command name isn't `"onStatus"`.
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
    ///             Number,
    ///             AmfString,
    ///             Object,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("onStatus"));
    /// buffer.encode(&Number::from(0u8));
    /// buffer.encode(&Null);
    /// buffer.encode(&Object::default());
    /// assert!(Decoder::<OnStatus>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// buffer.encode(&Number::from(0u8));
    /// buffer.encode(&Null);
    /// buffer.encode(&Object::default());
    /// assert!(Decoder::<OnStatus>::decode(&mut buffer).is_err())
    /// ```
    fn decode(&mut self) -> IOResult<OnStatus> {
        Decoder::<AmfString>::decode(self).and_then(
            |command| ensure_command_name("onStatus", command)
        )?;

        Decoder::<Number>::decode(self)?;
        Decoder::<Null>::decode(self)?;
        let info_object: Object = self.decode()?;

        Ok(OnStatus(info_object))
    }
}

impl Encoder<OnStatus> for ByteBuffer {
    /// Encodes an OnStatus command into bytes.
    fn encode(&mut self, on_status: &OnStatus) {
        self.encode(&AmfString::from(on_status.get_command_name()));
        self.encode(&on_status.get_transaction_id());
        self.encode(&Null);
        self.encode(on_status.get_info_object());
    }
}

#[cfg(test)]
mod tests {
    use crate::object;
    use super::*;

    #[test]
    fn decode_on_status() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("onStatus"));
        buffer.encode(&Number::from(0u8));
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
        let expected_transaction_id = f64::default();
        let expected_info_object = object!(
            "level" => AmfString::from("status"),
            "code" => AmfString::from("NetStream.Publish.Start"),
            "description" => AmfString::new(format!("{} is now published", "filename")),
            "details" => AmfString::from("filename")
        );
        buffer.encode(&OnStatus::new(expected_info_object.clone()));
        let command_name: AmfString = buffer.decode().unwrap();
        assert_eq!("onStatus", command_name);
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_info_object: Object = buffer.decode().unwrap();
        assert_eq!(expected_info_object, actual_info_object)
    }
}
