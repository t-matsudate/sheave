use std::{
    alloc::{
        GlobalAlloc,
        Layout,
        System
    },
    borrow::Cow,
    collections::HashMap,
    fmt::{
        Debug,
        Formatter,
        Result as FormatResult
    },
    io::Result as IOResult,
    ops::{
        Deref,
        DerefMut
    },
    sync::Arc
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer
};
use super::{
    *,
    super::{
        ensure_marker,
        invalid_string
    }
};

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnmarkedString(String);

#[doc(hidden)]
impl UnmarkedString {
    pub fn new(key: String) -> Self {
        Self(key)
    }
}

#[doc(hidden)]
impl Deref for UnmarkedString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[doc(hidden)]
impl DerefMut for UnmarkedString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[doc(hidden)]
impl<'a> PartialEq<&'a str> for UnmarkedString {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(other)
    }
}

#[doc(hidden)]
impl<'a> PartialEq<Cow<'a, str>> for UnmarkedString {
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.0.eq(other)
    }
}

#[doc(hidden)]
impl<'a> PartialEq<UnmarkedString> for &'a str {
    fn eq(&self, other: &UnmarkedString) -> bool {
        self.eq(&other.0)
    }
}

#[doc(hidden)]
impl<'a> PartialEq<UnmarkedString> for Cow<'a, str> {
    fn eq(&self, other: &UnmarkedString) -> bool {
        self.eq(&other.0)
    }
}

#[doc(hidden)]
impl PartialEq<UnmarkedString> for str {
    fn eq(&self, other: &UnmarkedString) -> bool {
        self.eq(&other.0)
    }
}

#[doc(hidden)]
impl PartialEq<String> for UnmarkedString {
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

#[doc(hidden)]
impl PartialEq<UnmarkedString> for String {
    fn eq(&self, other: &UnmarkedString) -> bool {
        self.eq(&other.0)
    }
}

#[doc(hidden)]
impl From<&str> for UnmarkedString {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

#[doc(hidden)]
impl Decoder<UnmarkedString> for ByteBuffer {
    fn decode(&mut self) -> IOResult<UnmarkedString> {
        let len = self.get_u16_be()? as usize;
        if len == 0 {
            return Ok("".into())
        }
        let bytes = self.get_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map(UnmarkedString::new).map_err(invalid_string)
    }
}

#[doc(hidden)]
impl Encoder<UnmarkedString> for ByteBuffer {
    fn encode(&mut self, string: &UnmarkedString) {
        assert!(string.len() <= u16::MAX as usize);
        self.put_u16_be(string.len() as u16);
        self.put_bytes(string.as_bytes());
    }
}

#[doc(hidden)]
pub struct Value {
    layout: Layout,
    ptr: *mut u8,
    marker: Marker
}

#[doc(hidden)]
impl Value {
    fn as_number(&self) -> &Number {
        unsafe {
            assert_eq!(Marker::Number, self.marker);
            &*self.ptr.cast::<Number>()
        }
    }

    fn as_boolean(&self) -> &Boolean {
        unsafe {
            assert_eq!(Marker::Boolean, self.marker);
            &*self.ptr.cast::<Boolean>()
        }
    }

    fn as_string(&self) -> &AmfString {
        unsafe {
            assert_eq!(Marker::AmfString, self.marker);
            &*self.ptr.cast::<AmfString>()
        }
    }

    fn as_object(&self) -> &Object {
        unsafe {
            assert_eq!(Marker::Object, self.marker);
            &*self.ptr.cast::<Object>()
        }
    }
}

#[doc(hidden)]
impl From<Number> for Value {
    fn from(value: Number) -> Self {
        unsafe {
            let layout = Layout::new::<Number>();
            let ptr = System.alloc(layout);
            ptr.cast::<Number>().write(value);
            Self {
                layout,
                ptr,
                marker: Marker::Number
            }
        }
    }
}

#[doc(hidden)]
impl From<Boolean> for Value {
    fn from(value: Boolean) -> Self {
        unsafe {
            let layout = Layout::new::<Boolean>();
            let ptr = System.alloc(layout);
            ptr.cast::<Boolean>().write(value);
            Self {
                layout,
                ptr,
                marker: Marker::Boolean
            }
        }
    }
}

#[doc(hidden)]
impl From<AmfString> for Value {
    fn from(value: AmfString) -> Self {
        unsafe {
            let layout = Layout::new::<AmfString>();
            let ptr = System.alloc(layout);
            ptr.cast::<AmfString>().write(value);
            Self {
                layout,
                ptr,
                marker: Marker::AmfString
            }
        }
    }
}

#[doc(hidden)]
impl From<Object> for Value {
    fn from(value: Object) -> Self {
        unsafe {
            let layout = Layout::new::<Object>();
            let ptr = System.alloc(layout);
            ptr.cast::<Object>().write(value);
            Self {
                layout,
                ptr,
                marker: Marker::Object
            }
        }
    }
}

#[doc(hidden)]
impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            System.dealloc(self.ptr, self.layout);
        }
    }
}

#[doc(hidden)]
impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        match self.marker {
            Marker::Number => Debug::fmt(self.as_number(), f),
            Marker::Boolean => Debug::fmt(self.as_boolean(), f),
            Marker::AmfString => Debug::fmt(self.as_string(), f),
            Marker::Object => Debug::fmt(self.as_object(), f),
            _ => unimplemented!("Debugging other types.")
        }
    }
}

#[doc(hidden)]
impl PartialEq<Self> for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.marker != other.marker {
            false
        } else {
            match self.marker {
                Marker::Number => PartialEq::eq(self.as_number(), other.as_number()),
                Marker::Boolean => PartialEq::eq(self.as_number(), other.as_number()),
                Marker::AmfString => PartialEq::eq(self.as_string(), other.as_string()),
                Marker::Object => PartialEq::eq(self.as_object(), other.as_object()),
                _ => unimplemented!("Comparing other types.")
            }
        }
    }
}

#[doc(hidden)]
impl Eq for Value {}

#[doc(hidden)]
impl Decoder<Value> for ByteBuffer {
    fn decode(&mut self) -> IOResult<Value> {
        let marker: Marker = self.peek_u8()?.into();

        match marker {
            Marker::Number => Decoder::<Number>::decode(self).map(Value::from),
            Marker::Boolean => Decoder::<Boolean>::decode(self).map(Value::from),
            Marker::AmfString => Decoder::<AmfString>::decode(self).map(Value::from),
            Marker::Object => Decoder::<Object>::decode(self).map(Value::from),
            _ => unimplemented!("Decoding other types.")
        }
    }
}

#[doc(hidden)]
impl Encoder<Value> for ByteBuffer {
    fn encode(&mut self, value: &Value) {
        match value.marker {
            Marker::Number => self.encode(value.as_number()),
            Marker::Boolean => self.encode(value.as_boolean()),
            Marker::AmfString => self.encode(value.as_string()),
            Marker::Object => self.encode(value.as_object()),
            _ => unimplemented!("Encoding other types.")
        }
    }
}

/// The anonymous object type of AMF.
/// This consists of pairs of string keys and any AMF data types.
///
/// * Key
///
/// The string which doesn't have its marker.
/// This type is named as `UnmarkedString` in this crate.
/// Also this occurs the panic if its length exceeds the range of 16 bits.
///
/// * Value
///
/// The pointer for AMF data types, which is wrapped into `Arc`.
/// This is because of avoiding to be deallocated its value unexpectedly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Object(HashMap<UnmarkedString, Arc<Value>>);

impl Object {
    /// Constrcuts a new object.
    /// 
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use sheave_core::messages::amf::v0::Object;
    ///
    /// Object::new(HashMap::default());
    /// ```
    pub fn new(object: HashMap<UnmarkedString, Arc<Value>>) -> Self {
        Self(object)
    }

    /// Insert a pair.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::{
    ///     AmfString,
    ///     Object
    /// };
    ///
    /// let mut object = Object::default();
    /// object.insert("app", AmfString::from("ondemand"))
    /// ```
    pub fn insert<V: Into<Value>>(&mut self, key: &str, value: V) {
        self.0.insert(key.into(), Arc::new(value.into()));
    }
}

impl Decoder<Object> for ByteBuffer {
    /// Decodes bytes into an AMF's Object type.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When buffer isn't remained at least 2 bytes. (non-empty object contains at least one pair of key and value)
    ///
    /// * [`InconsistentMarker`]
    ///
    /// When a marker byte doesn't indicate the AMF Object.
    ///
    /// * [`InvalidString`]
    ///
    /// When key bytes are invalid for a UTF-8 string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     messages::amf::v0::{
    ///         Marker,
    ///         Object
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Object as u8);
    /// // AMF's Object type is required a marker of object end (0x09) which is associated with an empty key.
    /// buffer.put_u16_be(0);
    /// buffer.put_u8(Marker::ObjectEnd as u8);
    /// assert!(Decoder::<Object>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Number as u8);
    /// buffer.put_u16_be(0);
    /// buffer.put_u8(Marker::ObjectEnd as u8);
    /// assert!(Decoder::<Object>::decode(&mut buffer).is_err());
    ///
    /// // This is a missing sequence of the "sparkle heart(💖)".
    /// let mut bytes = vec![0, 159, 146, 150];
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Object as u8);
    /// buffer.put_u16_be(4);
    /// buffer.put_bytes(&bytes);
    /// buffer.put_u8(Marker::Number as u8);
    /// buffer.put_f64(0.0);
    /// buffer.put_u16_be(0);
    /// buffer.put_u8(Marker::ObjectEnd as u8);
    /// assert!(Decoder::<Object>::decode(&mut buffer).is_err());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<Object>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::errors::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::errors::InconsistentMarker
    /// [`InvalidString`]: crate::errors::InvalidString
    fn decode(&mut self) -> IOResult<Object> {
        self.get_u8().and_then(
            |marker| ensure_marker(Marker::Object as u8, marker)
        )?;

        let mut m: HashMap<UnmarkedString, Arc<Value>> = HashMap::new();
        loop {
            let key: UnmarkedString = self.decode()?;

            if key == "" {
                self.get_u8().and_then(
                    |marker| ensure_marker(Marker::ObjectEnd as u8, marker)
                )?;

                return Ok(Object(m))
            } else {
                let value: Value = self.decode()?;
                m.insert(key, Arc::new(value));
            }
        }
    }
}

impl Encoder<Object> for ByteBuffer {
    /// Encodes an AMF's Object into bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Encoder,
    ///     messages::amf::v0::{
    ///         Marker,
    ///         Object
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Object::default());
    /// let bytes: Vec<u8> = buffer.into();
    /// assert_eq!(Marker::Object as u8, bytes[0]);
    /// assert_eq!(0u16.to_be_bytes().as_slice(), &bytes[1..3]);
    /// assert_eq!(Marker::ObjectEnd as u8, bytes[3])
    /// ```
    fn encode(&mut self, object: &Object) {
        self.put_u8(Marker::Object as u8);
        for (k, v) in &object.0 {
            self.encode(k);
            self.encode(v.as_ref());
        }
        self.encode(&UnmarkedString::from(""));
        self.put_u8(Marker::ObjectEnd as u8);
    }
}

/// Constructs an AMF's Object.
///
/// # Examples
///
/// ```rust
/// use sheave_core::{
///     // Note the macro is exported from the top of crate.
///     object,
///     messages::amf::v0::{
///         Number,
///         AmfString,
///         Object
///     }
/// };
///
/// let mut command_object = Object::default();
/// command_object.insert("app", AmfString::from("ondemand"));
/// command_object.insert("type", AmfString::from("nonprivate"));
/// command_object.insert("flashVer", AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"));
/// command_object.insert("tcUrl", AmfString::from("rtmp://localhost"));
/// assert_eq!(
///     command_object,
///     object!(
///         "app" => AmfString::from("ondemand"),
///         "type" => AmfString::from("nonprivate"),
///         "flashVer" => AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"),
///         "tcUrl" => AmfString::from("rtmp://localhost")
///     )
/// )
/// ```
#[macro_export]
macro_rules! object {
    ($($key:expr => $value:expr),*) => {
        {
            use $crate::messages::amf::v0::Object;
            let mut object = Object::default();
            $(object.insert($key, $value);)*
            object
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_value() {
        let number = Number::new(0f64);
        let allocated = Value::from(number);
        assert_eq!(number, *allocated.as_number())
    }

    #[test]
    fn boolean_value() {
        let boolean = Boolean::new(0u8);
        let allocated = Value::from(boolean);
        assert_eq!(boolean, *allocated.as_boolean())
    }

    #[test]
    fn string_value() {
        let string = AmfString::new("".into());
        let allocated = Value::from(string.clone());
        assert_eq!(string, *allocated.as_string())
    }

    #[test]
    fn object_value() {
        let object = Object::default();
        let allocated = Value::from(object.clone());
        assert_eq!(object, *allocated.as_object());
    }

    #[test]
    fn decode_unmarked_string() {
        let mut buffer = ByteBuffer::default();
        let app_key = "app";
        buffer.put_u16_be(app_key.len() as u16);
        buffer.put_bytes(app_key.as_bytes());
        let result: IOResult<UnmarkedString> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(app_key, actual);

        let mut buffer = ByteBuffer::default();
        let type_key = "type";
        buffer.put_u16_be(type_key.len() as u16);
        buffer.put_bytes(type_key.as_bytes());
        let result: IOResult<UnmarkedString> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(type_key, actual);

        let mut buffer = ByteBuffer::default();
        let flash_ver_key = "flashVer";
        buffer.put_u16_be(flash_ver_key.len() as u16);
        buffer.put_bytes(flash_ver_key.as_bytes());
        let result: IOResult<UnmarkedString> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!("flashVer", actual);

        let mut buffer = ByteBuffer::default();
        let tc_url_key = "tcUrl";
        buffer.put_u16_be(tc_url_key.len() as u16);
        buffer.put_bytes(tc_url_key.as_bytes());
        let result: IOResult<UnmarkedString> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(tc_url_key, actual);

        let mut buffer = ByteBuffer::default();
        let object_end_key = "";
        buffer.put_u16_be(object_end_key.len() as u16);
        buffer.put_bytes(object_end_key.as_bytes());
        let result: IOResult<UnmarkedString> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(object_end_key, actual)
    }

    #[test]
    fn encode_unmarked_string() {
        let mut buffer = ByteBuffer::default();
        let app_key = "app";
        buffer.encode(&UnmarkedString::from(app_key));
        let result: Vec<u8> = buffer.into();
        assert_eq!(&(app_key.len() as u16).to_be_bytes(), &result[..2]);
        assert_eq!(app_key.as_bytes(), &result[2..]);

        let mut buffer = ByteBuffer::default();
        let type_key = "type";
        buffer.encode(&UnmarkedString::from(type_key));
        let result: Vec<u8> = buffer.into();
        assert_eq!(&(type_key.len() as u16).to_be_bytes(), &result[..2]);
        assert_eq!(type_key.as_bytes(), &result[2..]);

        let mut buffer = ByteBuffer::default();
        let flash_ver_key = "flashVer";
        buffer.encode(&UnmarkedString::from(flash_ver_key));
        let result: Vec<u8> = buffer.into();
        assert_eq!(&(flash_ver_key.len() as u16).to_be_bytes(), &result[..2]);
        assert_eq!(flash_ver_key.as_bytes(), &result[2..]);

        let mut buffer = ByteBuffer::default();
        let tc_url_key = "tcUrl";
        buffer.encode(&UnmarkedString::from(tc_url_key));
        let result: Vec<u8> = buffer.into();
        assert_eq!(&(tc_url_key.len() as u16).to_be_bytes(), &result[..2]);
        assert_eq!(tc_url_key.as_bytes(), &result[2..])
    }

    #[test]
    fn decode_value() {
        let mut buffer = ByteBuffer::default();
        let ondemand = "ondemand";
        buffer.put_u8(Marker::AmfString as u8);
        buffer.put_u16_be(ondemand.len() as u16);
        buffer.put_bytes(ondemand.as_bytes());
        let result: IOResult<Value> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(Value::from(AmfString::from(ondemand)), actual);

        let mut buffer = ByteBuffer::default();
        let nonprivate = "nonprivate";
        buffer.put_u8(Marker::AmfString as u8);
        buffer.put_u16_be(nonprivate.len() as u16);
        buffer.put_bytes(nonprivate.as_bytes());
        let result: IOResult<Value> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(Value::from(AmfString::from(nonprivate)), actual);

        let mut buffer = ByteBuffer::default();
        let fmle = "FMLE/3.0 (compatible; Lavf 60.10.100)";
        buffer.put_u8(Marker::AmfString as u8);
        buffer.put_u16_be(fmle.len() as u16);
        buffer.put_bytes(fmle.as_bytes());
        let result: IOResult<Value> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(Value::from(AmfString::from(fmle)), actual);

        let mut buffer = ByteBuffer::default();
        let localhost = "rtmp://localhost";
        buffer.put_u8(Marker::AmfString as u8);
        buffer.put_u16_be(localhost.len() as u16);
        buffer.put_bytes(localhost.as_bytes());
        let result: IOResult<Value> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(Value::from(AmfString::from(localhost)), actual)
    }

    #[test]
    fn encode_value() {
        let mut buffer = ByteBuffer::default();
        let ondemand = "ondemand";
        buffer.encode(&Value::from(AmfString::from(ondemand)));
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::AmfString as u8, result[0]);
        assert_eq!(&(ondemand.len() as u16).to_be_bytes(), &result[1..3]);
        assert_eq!(ondemand.as_bytes(), &result[3..]);

        let mut buffer = ByteBuffer::default();
        let nonprivate = "nonprivate";
        buffer.encode(&Value::from(AmfString::from(nonprivate)));
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::AmfString as u8, result[0]);
        assert_eq!(&(nonprivate.len() as u16).to_be_bytes(), &result[1..3]);
        assert_eq!(nonprivate.as_bytes(), &result[3..]);

        let mut buffer = ByteBuffer::default();
        let fmle = "FMLE/3.0 (compatible; Lavf 60.10.100)";
        buffer.encode(&Value::from(AmfString::from(fmle)));
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::AmfString as u8, result[0]);
        assert_eq!(&(fmle.len() as u16).to_be_bytes(), &result[1..3]);
        assert_eq!(fmle.as_bytes(), &result[3..]);

        let mut buffer = ByteBuffer::default();
        let localhost = "rtmp://localhost";
        buffer.encode(&Value::from(AmfString::from(localhost)));
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::AmfString as u8, result[0]);
        assert_eq!(&(localhost.len() as u16).to_be_bytes(), &result[1..3]);
        assert_eq!(localhost.as_bytes(), &result[3..])
    }

    #[test]
    fn decode_object() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u8(Marker::Object as u8);
        buffer.encode(&UnmarkedString::from(""));
        buffer.put_u8(Marker::ObjectEnd as u8);
        let result: IOResult<Object> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(Object::default(), actual)
    }

    #[test]
    fn encode_object() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Object::default());
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::Object as u8, result[0]);
        assert_eq!(&0u16.to_be_bytes(), &result[1..3]);
        assert_eq!(Marker::ObjectEnd as u8, result[3])
    }
}
