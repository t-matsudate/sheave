mod invalid_publishing_type;

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
            Null
        },
        ensure_command_name,
        headers::MessageType
    }
};
pub use self::invalid_publishing_type::*;

/// The command to tell publishing information.
#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    transaction_id: Number,
    publishing_name: AmfString,
    publishing_type: AmfString
}

impl Publish {
    const COMMAND_NAME: &'static str = "publish";

    /// Constructs a Publish command.
    pub fn new(transaction_id: Number, publishing_name: AmfString, publishing_type: AmfString) -> Self {
        Self { transaction_id, publishing_name, publishing_type }
    }

    /// Gets the publishing identifier. (e.g. filename, username, etc.)
    pub fn get_publishing_name(&self) -> &AmfString {
        &self.publishing_name
    }

    /// Gets one of publishing type which is either `"live"`, `"record"` or `"append"`.
    pub fn get_publishing_type(&self) -> &AmfString {
        &self.publishing_type
    }
}

impl From<Publish> for (AmfString, AmfString) {
    fn from(publish: Publish) -> Self {
        (publish.publishing_name, publish.publishing_type)
    }
}

impl ChunkData for Publish {
    const CHANNEL: Channel = Channel::Source;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for Publish {
    fn get_command_name(&self) -> &str {
        Self::COMMAND_NAME
    }

    fn get_transaction_id(&self) -> Number {
        self.transaction_id
    }
}

impl Decoder<Publish> for ByteBuffer {
    /// Decodes bytes into a Publish command.
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
    /// When the command name isn't `"publish"`.
    ///
    /// * [`InvalidPublishingType`]
    ///
    /// When the publishing type is neither `"live"`, `"record"` nor `"append"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         Publish,
    ///         amf::v0::{
    ///             Number,
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("publish"));
    /// buffer.encode(&Number::new(5f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// buffer.encode(&AmfString::from("live"));
    /// assert!(Decoder::<Publish>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// buffer.encode(&Number::new(5f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// buffer.encode(&AmfString::from("live"));
    /// assert!(Decoder::<Publish>::decode(&mut buffer).is_err());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("publish"));
    /// buffer.encode(&Number::new(5f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// buffer.encode(&AmfString::from("something else"));
    /// assert!(Decoder::<Publish>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    /// [`InvalidPublishingType`]: InvalidPublishingType
    fn decode(&mut self) -> IOResult<Publish> {
        Decoder::<AmfString>::decode(self).and_then(
            |command| ensure_command_name("publish", command)
        )?;

        let transaction_id: Number = self.decode()?;
        Decoder::<Null>::decode(self)?;
        let publishing_name: AmfString = self.decode()?;
        let publishing_type: AmfString = self.decode()?;

        if publishing_type != "live" && publishing_type != "record" && publishing_type != "append" {
            return Err(invalid_publishing_type(publishing_type))
        }

        Ok(Publish { transaction_id, publishing_name, publishing_type })
    }
}

impl Encoder<Publish> for ByteBuffer {
    /// Encodes a Publish command into bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         Publish,
    ///         amf::v0::{
    ///             Number,
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Publish::new(5.into(), AmfString::default(), "live".into()));
    /// let command_name: AmfString = buffer.decode().unwrap();
    /// let transaction_id: Number = buffer.decode().unwrap();
    /// Decoder::<Null>::decode(&mut buffer).unwrap();
    /// let publishing_name: AmfString = buffer.decode().unwrap();
    /// let publishing_type: AmfString = buffer.decode().unwrap();
    /// assert_eq!("publish", command_name);
    /// assert_eq!(5f64, transaction_id);
    /// assert_eq!(AmfString::default(), publishing_name);
    /// assert_eq!("live", publishing_type)
    /// ```
    fn encode(&mut self, publish: &Publish) {
        self.encode(&AmfString::from(publish.get_command_name()));
        self.encode(&publish.get_transaction_id());
        self.encode(&Null);
        self.encode(publish.get_publishing_name());
        self.encode(publish.get_publishing_type());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_publish() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("publish"));
        buffer.encode(&Number::new(5f64));
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());
        buffer.encode(&AmfString::from("live"));
        let result: IOResult<Publish> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = Publish::new(5.into(), AmfString::default(), AmfString::from("live"));
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_publish() {
        let mut buffer = ByteBuffer::default();
        let expected_transaction_id = 5f64;
        let expected_publishing_name = "";
        let expected_publishing_type = "live";
        let expected = Publish::new(Number::new(expected_transaction_id), AmfString::from(expected_publishing_name), AmfString::from(expected_publishing_type));
        buffer.encode(&expected);
        let command_name: AmfString = buffer.decode().unwrap();
        assert_eq!("publish", command_name);
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_publishing_name: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_publishing_name, actual_publishing_name);
        let actual_publishing_type: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_publishing_type, actual_publishing_type)
    }
}
