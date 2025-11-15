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
            AmfString,
            Null
        },
        headers::MessageType
    }
};
pub use self::invalid_publishing_type::*;

/// The command to tell publishing information.
///
/// Following format is required:
///
/// |Field|AMF Type|Value|
/// | :- | :- | :- |
/// ||[`Null`]|Nothing but an AMF's type marker is in.|
/// |Publishing Name|[`String`]|A name for publishing a data to the server.|
/// |Publishing Type|[`String`]|See [Publishing Type](#publishing-type).|
///
/// # Publishing Type
///
/// The publish command requires you to specify one of "Publishing Type" in its request.
/// Publishing Type means:
///
/// |Pubishing Type|Description|
/// | :- | :- |
/// |`"live"`|Only streaming.<br />Its data will never be stored.|
/// |`"record"`|Its data will be stored.<br />If publishing name duplicated, it is rewritten as a new file.|
/// |`"append"`|Same as `"record"` excepts is appended its data if publishing name duplicated.|
///
/// [`String`]: crate::messages::amf::v0::AmfString
/// [`Null`]: crate::messages::amf::v0::Null
#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    publishing_name: AmfString,
    publishing_type: AmfString
}

impl Publish {
    /// Constructs a Publish command.
    pub fn new(publishing_name: AmfString, publishing_type: AmfString) -> Self {
        Self { publishing_name, publishing_type }
    }

    /// Gets the publishing identifier. (e.g. filename)
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

impl Command for Publish {}

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
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// buffer.encode(&AmfString::from("live"));
    /// assert!(Decoder::<Publish>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// buffer.encode(&AmfString::from("record"));
    /// assert!(Decoder::<Publish>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// buffer.encode(&AmfString::from("append"));
    /// assert!(Decoder::<Publish>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// buffer.encode(&AmfString::from("something else"));
    /// assert!(Decoder::<Publish>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InvalidPublishingType`]: InvalidPublishingType
    fn decode(&mut self) -> IOResult<Publish> {
        Decoder::<Null>::decode(self)?;
        let publishing_name: AmfString = self.decode()?;
        let publishing_type: AmfString = self.decode()?;

        if publishing_type != "live" && publishing_type != "record" && publishing_type != "append" {
            return Err(invalid_publishing_type(publishing_type))
        }

        Ok(Publish { publishing_name, publishing_type })
    }
}

impl Encoder<Publish> for ByteBuffer {
    /// Encodes a Publish command into bytes.
    fn encode(&mut self, publish: &Publish) {
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
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());
        buffer.encode(&AmfString::from("live"));
        let result: IOResult<Publish> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = Publish::new(AmfString::default(), AmfString::from("live"));
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_publish() {
        let mut buffer = ByteBuffer::default();
        let expected_publishing_name = "";
        let expected_publishing_type = "live";
        let expected = Publish::new(AmfString::from(expected_publishing_name), AmfString::from(expected_publishing_type));
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_publishing_name: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_publishing_name, actual_publishing_name);
        let actual_publishing_type: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_publishing_type, actual_publishing_type)
    }
}
