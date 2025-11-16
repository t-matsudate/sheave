use std::{
    borrow::Cow,
    error::Error,
    ffi::OsString,
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
    path::{
        Path,
        PathBuf
    },
    rc::Rc,
    string::String as StdString,
    sync::Arc,
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

impl PartialEq<str> for AmfString {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<AmfString> for str {
    fn eq(&self, other: &AmfString) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<&'a str> for AmfString {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<AmfString> for &'a str {
    fn eq(&self, other: &AmfString) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, str>> for AmfString {
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<AmfString> for Cow<'a, str> {
    fn eq(&self, other: &AmfString) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<Path> for AmfString {
    fn eq(&self, other: &Path) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<AmfString> for Path {
    fn eq(&self, other: &AmfString) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<PathBuf> for AmfString {
    fn eq(&self, other: &PathBuf) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<AmfString> for PathBuf {
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

impl From<&AmfString> for AmfString {
    fn from(s: &AmfString) -> Self {
        s.clone()
    }
}

impl From<char> for AmfString {
    fn from(c: char) -> Self {
        Self(String::from(c))
    }
}

impl From<&str> for AmfString {
    fn from(s: &str) -> Self {
        Self(String::from(s))
    }
}

impl From<&mut str> for AmfString {
    fn from(s: &mut str) -> Self {
        Self(String::from(s))
    }
}

impl From<AmfString> for OsString {
    fn from(s: AmfString) -> Self {
        Self::from(s.0)
    }
}

impl From<AmfString> for PathBuf {
    fn from(s: AmfString) -> Self {
        Self::from(s.0)
    }
}

impl From<AmfString> for Rc<str> {
    fn from(v: AmfString) -> Self {
        Self::from(v.0)
    }
}

impl From<AmfString> for Arc<str> {
    fn from(v: AmfString) -> Self {
        Self::from(v.0)
    }
}

impl From<AmfString> for Vec<u8> {
    fn from(s: AmfString) -> Self {
        Self::from(s.0)
    }
}

impl<'a> From<AmfString> for Box<dyn Error + 'a> {
    fn from(str_err: AmfString) -> Self {
        Self::from(str_err.0)
    }
}

impl<'a> From<AmfString> for Box<dyn Error + Send + Sync + 'a> {
    fn from(str_err: AmfString) -> Self {
        Self::from(str_err.0)
    }
}

impl<'a> From<Cow<'a, str>> for AmfString {
    fn from(s: Cow<'a, str>) -> Self {
        Self(String::from(s))
    }
}

impl<'a> From<AmfString> for Cow<'a, str> {
    fn from(s: AmfString) -> Self {
        Self::from(s.0)
    }
}

impl<'a> From<&'a AmfString> for Cow<'a, str> {
    fn from(s: &'a AmfString) -> Self {
        Self::from(&s.0)
    }
}

impl From<Box<str>> for AmfString {
    fn from(s: Box<str>) -> Self {
        Self(String::from(s))
    }
}

impl From<AmfString> for Box<str> {
    fn from(s: AmfString) -> Self {
        Self::from(s.0)
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
