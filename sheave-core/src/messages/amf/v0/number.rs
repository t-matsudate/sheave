use std::{
    cmp::Ordering,
    io::Result as IOResult,
    ops::{
        Add,
        AddAssign
    }
};
use super::{
    Marker,
    super::ensure_marker
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer
};

/// The IEEE 754 double precision floating point number of AMF data types.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Number(f64);

impl Number {
    /// Constructs an AMF's Number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// Number::new(0f64);
    /// ```
    pub fn new(number: f64) -> Self {
        Self(number)
    }
}

impl From<bool> for Number {
    /// Converts a bool value into an AMF's Number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert_eq!(Number::new(0.0), Number::from(false));
    /// assert_eq!(Number::new(1.0), Number::from(true))
    /// ```
    fn from(number: bool) -> Self {
        Self(number.into())
    }
}

impl From<f32> for Number {
    /// Converts an IEEE 754 single precision floating point number into an AMF's Number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert_eq!(Number::new(0.0), Number::from(0.0))
    /// ```
    fn from(number: f32) -> Self {
        Self(number.into())
    }
}

impl From<i8> for Number {
    /// Converts a signed 1 byte integer into an AMF's Number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert_eq!(Number::new(0.0), Number::from(0i8))
    /// ```
    fn from(number: i8) -> Self {
        Self(number.into())
    }
}

impl From<i16> for Number {
    /// Converts a signed 2 bytes integer into an AMF's Number.
    ///
    /// # Examples
    /// 
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert_eq!(Number::new(0f64), Number::from(0i16))
    /// ```
    fn from(number: i16) -> Self {
        Self(number.into())
    }
}

impl From<i32> for Number {
    /// Converts a signed 4 bytes integer into an AMF's Number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert_eq!(Number::new(0f64), Number::from(0i32))
    /// ```
    fn from(number: i32) -> Self {
        Self(number.into())
    }
}

impl From<u8> for Number {
    /// Converts an unsigned 1 byte integer into an AMF's Number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert_eq!(Number::new(0f64), Number::from(0u8))
    /// ```
    fn from(number: u8) -> Self {
        Self(number.into())
    }
}

impl From<u16> for Number {
    /// Converts an unsigned 2 bytes integer into an AMF's Number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert_eq!(Number::new(0f64), Number::from(0u16))
    /// ```
    fn from(number: u16) -> Self {
        Self(number.into())
    }
}

impl From<u32> for Number {
    /// Converts an unsigned 4 bytes integer into an AMF's Number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert_eq!(Number::new(0f64), Number::from(0u32))
    /// ```
    fn from(number: u32) -> Self {
        Self(number.into())
    }
}

impl PartialEq<f64> for Number {
    /// Checks whether this equals an other value, as the IEEE 754 double precision floating point number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert!(Number::new(0.0) == 0.0)
    /// ```
    fn eq(&self, other: &f64) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<f64> for Number {
    /// Compares this with an other value, as the IEEE 754 double precision floating point number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert!(Number::new(0.0) < 1.0)
    /// ```
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<Number> for f64 {
    /// Makes two values commutative.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert!(0.0 == Number::new(0.0))
    /// ```
    fn eq(&self, other: &Number) -> bool {
        self.eq(&other.0)
    }
}

impl PartialOrd<Number> for f64 {
    /// Makes two values commutative.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::v0::Number;
    ///
    /// assert!(0.0 < Number::new(1.0))
    /// ```
    fn partial_cmp(&self, other: &Number) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Add for Number {
    type Output = Number;

    fn add(self, rhs: Self) -> Self::Output {
        Number(self.0 + rhs.0)
    }
}

impl Add<f64> for Number {
    type Output = Number;

    fn add(self, rhs: f64) -> Self::Output {
        Number(self.0 + rhs)
    }
}

impl AddAssign for Number {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<f64> for Number {
    fn add_assign(&mut self, rhs: f64) {
        self.0 += rhs;
    }
}

impl Decoder<Number> for ByteBuffer {
    /// Decodes bytes into an AMF's Number.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When buffer isn't remained at least 9 bytes.
    ///
    /// * [`InconsistentMarker`]
    ///
    /// When a marker byte doesn't indicate the AMF Number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::random;
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     messages::amf::v0::{
    ///         Marker,
    ///         Number
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Number as u8);
    /// buffer.put_f64(f64::from_bits(random::<u64>()));
    /// assert!(Decoder::<Number>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Boolean as u8);
    /// buffer.put_f64(f64::from_bits(random::<u64>()));
    /// assert!(Decoder::<Number>::decode(&mut buffer).is_err());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<Number>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<Number> {
        self.get_u8().and_then(
            |marker| ensure_marker(Marker::Number as u8, marker)
        )?;

        self.get_f64().map(Number::new)
    }
}

impl Encoder<Number> for ByteBuffer {
    /// Encodes an AMF's Number into bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Encoder,
    ///     messages::amf::v0::{
    ///         Marker,
    ///         Number
    ///     }
    /// };
    ///
    /// let n: f64 = 0.0;
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Number::new(n));
    /// let bytes: Vec<u8> = buffer.into();
    /// assert_eq!(Marker::Number as u8, bytes[0]);
    /// assert_eq!(n.to_be_bytes().as_slice(), &bytes[1..])
    /// ```
    fn encode(&mut self, n: &Number) {
        self.put_u8(Marker::Number as u8);
        self.put_f64(n.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_number() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u8(Marker::Number as u8);
        buffer.put_f64(1f64);
        let result: IOResult<Number> = buffer.decode();
        assert!(result.is_ok());
        let number = result.unwrap();
        assert_eq!(1f64, number)
    }

    #[test]
    fn encode_number() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Number::new(1f64));
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::Number as u8, result[0]);
        assert_eq!(&1f64.to_be_bytes(), &result[1..])
    }
}
