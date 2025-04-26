//! # The AMF Data Types (version 0).
//!
//! These are data types which are defined in the Action Message Format version 0 specification.
//! Currently the RTMP uses following types:
//!
//! |Marker|AMF Data Type|Description|
//! | -: | :- | :- |
//! |`0`|[`Number`]|The IEEE 754 double precision floating point number.|
//! |`1`|`Boolean`|The boolean that is represented as a 1 byte data. (C-like)|
//! |`2`|[`AmfString`]|The string that is limited its length the range of 2 bytes.|
//! |`3`|[`Object`]|The key/value-paired object that its value type is flexible.|
//! |`5`|[`Null`]|Only the marker. Any value doesn't contain.|
//! |`8`|[`EcmaArray`]|Same as the [`Object`] except this has its length.|
//! |`9`|Object End|Indicates a stream of object terminaites there.|
//!
//! These are checked whether matched its marker with an actual data, by the receiver-side.
//! Therefore any stream which contains any unmatched marker will be considered it's invalid.
//! Note that a marker of the Object End must appear only in AMF's Object.
//! Following descriptions are the actual formats of AMF.
//!
//! ## [`Number`]
//!
//! 1. [`Marker`] (1 byte)
//! 2. An IEEE 754 double precision value (8 bytes)
//!
//! ## [`Boolean`]
//!
//! 1. [`Marker`] (1 byte)
//! 2. A boolean value (1 byte)
//!
//! ## [`String`]
//!
//! 1. [`Marker`] (1 byte)
//! 2. Length (2 bytes)
//! 3. UTF-8 characters (variable)
//!
//! ## [`Object`]
//!
//! 1. [`Marker`] (1 byte)
//! 2. Arbitrary number key-value pairs (variable)
//! 3. The Object End marker that is associated with an empty string.
//!
//! Note keys are without markers of the AMF String.
//! To be exact, these aren't defined as the AMF String in the specification, that is, these are just strings with lengths (within 2 bytes) at the beginning.
//!
//! Next, values are limited just in the AMF data types.
//! Currently, following AMF data types to be used in the RTMP are allowed to insert into the AMF Object type:
//!
//! * [`Number`]
//! * [`Boolean`]
//! * [`AmfString`]
//! * [`Object`]
//! * [`Null`]
//! * [`EcmaArray`]
//!
//! ## [`Null`]
//!
//! 1. [`Marker`] (1 byte)
//!
//! ## [`EcmaArray`]
//!
//! 1. [`Marker`] (1 byte)
//! 2. Length (4 bytes)
//! 3. Arbitrary number key-value pairs (variable)
//! 4. The Object End marker that is associated with an empty string.
//!
//! [`Marker`]: Marker
//! [`Number`]: Number
//! [`Boolean`]: Boolean
//! [`AmfString`]: AmfString
//! [`Object`]: Object
//! [`Null`]: Null
//! [`EcmaArray`]: EcmaArray

mod number;
mod boolean;
mod string;
mod object;
mod null;
mod ecma_array;

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
        DerefMut,
        Index
    },
    sync::Arc
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer,
    messages::amf::{
        ensure_marker,
        invalid_string
    }
};
pub use self::{
    number::Number,
    boolean::Boolean,
    string::AmfString,
    object::Object,
    null::Null,
    ecma_array::EcmaArray
};

/// Representation of markers of the AMF data types.
///
/// Variants correspond to respectively following numbers:
///
/// |Pattern|Number|
/// | :- | :- |
/// |`Number`|`0`|
/// |`Boolean`|`1`|
/// |`AmfString`|`2`|
/// |`Object`|`3`|
/// |`Null`|`5`|
/// |`EcmaArray`|`8`|
/// |`ObjectEnd`|`9`|
/// |`Other`|other numbers|
///
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    Number,
    Boolean,
    AmfString,
    Object,
    Null = 0x05,
    EcmaArray = 0x08,
    ObjectEnd = 0x09,
    Other = 0xff
}

impl From<u8> for Marker {
    fn from(marker: u8) -> Self {
        use Marker::*;

        match marker {
            0 => Number,
            1 => Boolean,
            2 => AmfString,
            3 => Object,
            5 => Null,
            8 => EcmaArray,
            9 => ObjectEnd,
            _ => Other
        }
    }
}

impl From<Marker> for u8 {
    fn from(marker: Marker) -> Self {
        marker as u8
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnmarkedString(String);

#[doc(hidden)]
impl UnmarkedString {
    pub(self) fn new(key: String) -> Self {
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

    fn as_null(&self) -> &Null {
        unsafe {
            assert_eq!(Marker::Null, self.marker);
            &*self.ptr.cast::<Null>()
        }
    }

    fn as_object(&self) -> &Object {
        unsafe {
            assert_eq!(Marker::Object, self.marker);
            &*self.ptr.cast::<Object>()
        }
    }

    fn as_ecma_array(&self) -> &EcmaArray {
        unsafe {
            assert_eq!(Marker::EcmaArray, self.marker);
            &*self.ptr.cast::<EcmaArray>()
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
impl<'a> From<&'a Value> for &'a Number {
    fn from(value: &'a Value) -> Self {
        value.as_number()
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
impl<'a> From<&'a Value> for &'a AmfString {
    fn from(value: &'a Value) -> Self {
        value.as_string()
    }
}

#[doc(hidden)]
impl From<Null> for Value {
    fn from(value: Null) -> Self {
        unsafe {
            let layout = Layout::new::<Null>();
            let ptr = System.alloc(layout);
            ptr.cast::<Null>().write(value);
            Self {
                layout,
                ptr,
                marker: Marker::Null
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
impl From<EcmaArray> for Value {
    fn from(value: EcmaArray) -> Self {
        unsafe {
            let layout = Layout::new::<EcmaArray>();
            let ptr = System.alloc(layout);
            ptr.cast::<EcmaArray>().write(value);
            Self {
                layout,
                ptr,
                marker: Marker::EcmaArray
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
            Marker::Null => Debug::fmt(self.as_null(), f),
            Marker::Object => Debug::fmt(self.as_object(), f),
            Marker::EcmaArray => Debug::fmt(self.as_ecma_array(), f),
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
                Marker::Boolean => PartialEq::eq(self.as_boolean(), other.as_boolean()),
                Marker::AmfString => PartialEq::eq(self.as_string(), other.as_string()),
                Marker::Null => PartialEq::eq(self.as_null(), other.as_null()),
                Marker::Object => PartialEq::eq(self.as_object(), other.as_object()),
                Marker::EcmaArray => PartialEq::eq(self.as_ecma_array(), other.as_ecma_array()),
                _ => unimplemented!("Comparing other types.")
            }
        }
    }
}

#[doc(hidden)]
impl<T: Into<Value> + Clone> PartialEq<T> for Value {
    fn eq(&self, other: &T) -> bool {
        let value: Self = other.clone().into();
        self.eq(&value)
    }
}

#[doc(hidden)]
impl Eq for Value {}
#[doc(hidden)]
unsafe impl Send for Value {}
#[doc(hidden)]
unsafe impl Sync for Value {}

#[doc(hidden)]
impl Decoder<Value> for ByteBuffer {
    fn decode(&mut self) -> IOResult<Value> {
        let marker: Marker = self.peek_u8()?.into();

        match marker {
            Marker::Number => Decoder::<Number>::decode(self).map(Value::from),
            Marker::Boolean => Decoder::<Boolean>::decode(self).map(Value::from),
            Marker::AmfString => Decoder::<AmfString>::decode(self).map(Value::from),
            Marker::Null => Decoder::<Null>::decode(self).map(Value::from),
            Marker::Object => Decoder::<Object>::decode(self).map(Value::from),
            Marker::EcmaArray => Decoder::<EcmaArray>::decode(self).map(Value::from),
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
            Marker::Null => self.encode(value.as_null()),
            Marker::Object => self.encode(value.as_object()),
            Marker::EcmaArray => self.encode(value.as_ecma_array()),
            _ => unimplemented!("Encoding other types.")
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Properties(HashMap<UnmarkedString, Arc<Value>>);

#[doc(hidden)]
impl Properties {
    pub fn insert<V: Into<Value>>(&mut self, key: &str, value: V) {
        self.0.insert(key.into(), Arc::new(value.into()));
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(&key.into()).map(|value| &**value)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[doc(hidden)]
impl Index<&str> for Properties {
    type Output = Value;

    fn index(&self, key: &str) -> &Self::Output {
        &**self.0.index(&key.into())
    }
}

#[doc(hidden)]
impl Decoder<Properties> for ByteBuffer {
    fn decode(&mut self) -> IOResult<Properties> {
        let mut m: HashMap<UnmarkedString, Arc<Value>> = HashMap::new();
        loop {
            let key: UnmarkedString = self.decode()?;

            if key == "" {
                self.get_u8().and_then(
                    |marker| ensure_marker(Marker::ObjectEnd as u8, marker)
                )?;

                return Ok(Properties(m))
            } else {
                let value: Value = self.decode()?;
                m.insert(key, Arc::new(value));
            }
        }
    }
}

#[doc(hidden)]
impl Encoder<Properties> for ByteBuffer {
    fn encode(&mut self, properties: &Properties) {
        for (k, v) in &properties.0 {
            self.encode(k);
            self.encode(&**v);
        }
        self.encode(&UnmarkedString::from(""));
        self.put_u8(Marker::ObjectEnd as u8);
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
    fn null_value() {
        let null = Null;
        let allocated = Value::from(null);
        assert_eq!(null, *allocated.as_null())
    }

    #[test]
    fn object_value() {
        let object = Object::default();
        let allocated = Value::from(object.clone());
        assert_eq!(object, *allocated.as_object())
    }

    #[test]
    fn ecma_array_value() {
        let ecma_array = EcmaArray::default();
        let allocated = Value::from(ecma_array.clone());
        assert_eq!(ecma_array, *allocated.as_ecma_array());
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
        assert_eq!(app_key, actual)
    }

    #[test]
    fn encode_unmarked_string() {
        let mut buffer = ByteBuffer::default();
        let app_key = "app";
        buffer.encode(&UnmarkedString::from(app_key));
        let result: Vec<u8> = buffer.into();
        assert_eq!(&(app_key.len() as u16).to_be_bytes(), &result[..2]);
        assert_eq!(app_key.as_bytes(), &result[2..])
    }

    #[test]
    #[should_panic]
    fn panic_when_length_exceeded() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&UnmarkedString::new("a".repeat(1 + u16::MAX as usize)));
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
        assert_eq!(Value::from(AmfString::from(ondemand)), actual)
    }

    #[test]
    fn encode_value() {
        let mut buffer = ByteBuffer::default();
        let ondemand = "ondemand";
        buffer.encode(&Value::from(AmfString::from(ondemand)));
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::AmfString as u8, result[0]);
        assert_eq!(&(ondemand.len() as u16).to_be_bytes(), &result[1..3]);
        assert_eq!(ondemand.as_bytes(), &result[3..])
    }
}
