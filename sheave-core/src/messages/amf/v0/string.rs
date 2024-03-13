use std::{
    borrow::Cow,
    fmt::{
        Display,
        Formatter,
        Result as FormatResult
    },
    io::Result as IOResult,
    ops::{
        Deref,
        DerefMut
    },
    string::String as StdString
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer
};
use super::{
    Marker,
    super::{
        ensure_marker,
        invalid_string
    }
};

/// The UTF-8 string of AMF data types.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AmfString(StdString);

impl AmfString {
    /// Constructs an AMF's String.
    pub fn new(string: StdString) -> Self {
        Self(string)
    }
}

impl Deref for AmfString {
    type Target = StdString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AmfString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> PartialEq<&'a str> for AmfString {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<Cow<'a, str>> for AmfString {
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<AmfString> for &'a str {
    fn eq(&self, other: &AmfString) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<AmfString> for Cow<'a, str> {
    fn eq(&self, other: &AmfString) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<AmfString> for str {
    fn eq(&self, other: &AmfString) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<StdString> for AmfString {
    fn eq(&self, other: &StdString) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<AmfString> for StdString {
    fn eq(&self, other: &AmfString) -> bool {
        self.eq(&other.0)
    }
}

impl From<&str> for AmfString {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl Display for AmfString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        Display::fmt(&self.0, f)
    }
}

impl Decoder<AmfString> for ByteBuffer {
    /// Decodes bytes into an AMF's String.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When buffer isn't remained at least 3 bytes.
    ///
    /// * [`InconsistentMarker`]
    ///
    /// When a marker byte doesn't indicate the AMF String.
    ///
    /// * [`InvalidString`]
    ///
    /// When bytes are invalid for a UTF-8 string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     messages::amf::v0::{
    ///         Marker,
    ///         AmfString
    ///     }
    /// };
    ///
    /// let s = "hello world!".as_bytes();
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::AmfString as u8);
    /// buffer.put_u16_be(s.len() as u16);
    /// buffer.put_bytes(s);
    /// assert!(Decoder::<AmfString>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Number as u8);
    /// buffer.put_u16_be(s.len() as u16);
    /// buffer.put_bytes(s);
    /// assert!(Decoder::<AmfString>::decode(&mut buffer).is_err());
    ///
    /// // This is a missing sequence of the "sparkle heart(💖)".
    /// let bytes = vec![0, 159, 146, 150];
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::AmfString as u8);
    /// buffer.put_u16_be(bytes.len() as u16);
    /// buffer.put_bytes(&bytes);
    /// assert!(Decoder::<AmfString>::decode(&mut buffer).is_err());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<AmfString>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    fn decode(&mut self) -> IOResult<AmfString> {
        self.get_u8().and_then(
            |marker| ensure_marker(Marker::AmfString as u8, marker)
        )?;

        let len = self.get_u16_be()? as usize;
        if len == 0 {
            return Ok("".into())
        }
        let bytes = self.get_bytes(len)?;
        StdString::from_utf8(bytes.to_vec()).map(AmfString::new).map_err(invalid_string)
    }
}

impl Encoder<AmfString> for ByteBuffer {
    /// Encodes an AMF String into bytes.
    ///
    /// # Panics
    ///
    /// Its length must be the range of 16 bits.
    /// If it exceeds, a panic is occured.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::panic::catch_unwind;
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Encoder,
    ///     messages::amf::v0::{
    ///         Marker,
    ///         AmfString
    ///     }
    /// };
    ///
    /// let s = "hello world!";
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from(s));
    /// let bytes: Vec<u8> = buffer.into();
    /// assert_eq!(Marker::AmfString as u8, bytes[0]);
    /// assert_eq!((s.len() as u16).to_be_bytes().as_slice(), &bytes[1..3]);
    /// assert_eq!(s.as_bytes(), &bytes[3..]);
    ///
    /// let result = catch_unwind(
    ///     || {
    ///         let mut buffer = ByteBuffer::default();
    ///         buffer.encode(&AmfString::new("a".repeat(1 + u16::MAX as usize)))
    ///     }
    /// );
    /// assert!(result.is_err())
    /// ```
    fn encode(&mut self, string: &AmfString) {
        assert!(string.len() <= u16::MAX as usize);
        self.put_u8(Marker::AmfString as u8);
        self.put_u16_be(string.len() as u16);
        self.put_bytes(string.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_string() {
        let string = "connect".as_bytes();
        let mut buffer = ByteBuffer::default();
        buffer.put_u8(Marker::AmfString as u8);
        buffer.put_u16_be(string.len() as u16);
        buffer.put_bytes(string);
        let result: IOResult<AmfString> = buffer.decode();
        assert!(result.is_ok());
        let string = result.unwrap();
        assert_eq!("connect", string)
    }

    #[test]
    fn encode_string() {
        let string = AmfString::from("connect");
        let mut buffer = ByteBuffer::default();
        buffer.encode(&string);
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::AmfString as u8, result[0]);
        assert_eq!(&(string.len() as u16).to_be_bytes(), &result[1..3]);
        assert_eq!("connect".as_bytes(), &result[3..])
    }

    #[test]
    #[should_panic]
    fn panic_when_length_exceeded() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::new("a".repeat(1 + u16::MAX as usize)));
    }
}
