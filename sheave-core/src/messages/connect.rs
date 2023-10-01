use std::io::Result as IOResult;
use super::ensure_command_name;
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

/// The command to tell the information that where connects from/to.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Connect(Object);

impl Connect {
    /// Constructs a connect command.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::{
    ///     Connect,
    ///     amf::v0::Object
    /// };
    ///
    /// Connect::new(Object::default());
    /// ```
    pub fn new(command_object: Object) -> Self {
        Self(command_object)
    }

    /// Gets the transaction id.
    /// Note that must always be `1` in the Connect command.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::{
    ///     Connect,
    ///     amf::v0::Number
    /// };
    ///
    /// let connect = Connect::default();
    ///
    /// assert_eq!(Number::from(1), connect.get_transaction_id())
    /// ```
    pub fn get_transaction_id(&self) -> Number {
        Number::from(1)
    }

    /// Gets the command object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::{
    ///     Connect,
    ///     amf::v0::Object
    /// };
    ///
    /// let connect = Connect::default();
    ///
    /// assert_eq!(&Object::default(), connect.get_command_object());
    /// ```
    pub fn get_command_object(&self) -> &Object {
        &self.0
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
    /// * [`InvalidString`]
    ///
    /// When some value is invalid for UTF-8 string.
    ///
    /// * [`InconsistentCommand`]
    ///
    /// When the command name isn't `"connect"`.
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
    ///             Marker,
    ///             Number,
    ///             AmfString,
    ///             Object
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("connect"));
    /// buffer.encode(&Number::from(1));
    /// buffer.encode(&Object::default());
    /// assert!(Decoder::<Connect>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// buffer.encode(&Number::from(1));
    /// buffer.encode(&Object::default());
    /// assert!(Decoder::<Connect>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::errors::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::errors::InconsistentMarker
    /// [`InvalidString`]: crate::errors::InvalidString
    /// [`InconsistentCommand`]: crate::errors::InconsistentCommand
    fn decode(&mut self) -> IOResult<Connect> {
        Decoder::<AmfString>::decode(self).and_then(
            |command| ensure_command_name("connect", command)
        )?;

        // Skips because the transaction ID in the Connect command is fixed `1`.
        Decoder::<Number>::decode(self)?;
        let command_object: Object = self.decode()?;
        Ok(Connect(command_object))
    }
}

impl Encoder<Connect> for ByteBuffer {
    /// Encodes a Connect command into bytes.
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
    ///             AmfString,
    ///             Object
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Connect::default());
    /// let command_name: AmfString = buffer.decode().unwrap();
    /// let transaction_id: Number = buffer.decode().unwrap();
    /// let command_object: Object = buffer.decode().unwrap();
    /// assert_eq!("connect", command_name);
    /// assert_eq!(1f64, transaction_id);
    /// assert_eq!(Object::default(), command_object)
    /// ```
    fn encode(&mut self, connect: &Connect) {
        self.encode(&AmfString::from("connect"));
        self.encode(&connect.get_transaction_id());
        self.encode(connect.get_command_object());
    }
}

#[cfg(test)]
mod tests {
    use crate::object;
    use super::*;

    #[test]
    fn decode_connect_input() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("connect"));
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
        let command: AmfString = buffer.decode().unwrap();
        assert_eq!("connect", command);
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id);
        let actual_command_object: Object = buffer.decode().unwrap();
        assert_eq!(expected_command_object, actual_command_object)
    }
}