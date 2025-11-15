use std::io::Result as IOResult;
use log::warn;
use crate::{
    Decoder,
    Encoder,
    ByteBuffer,
    messages::amf::{
        ensure_marker,
        v0::Marker
    }
};
use super::Properties;

/// The **sized** object type of AMF.
///
/// This consists of:
///
/// * Count
///
/// The unsigned 32 bits integer.
///
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
///
/// This is expected its size is same as the above count.
///
/// You can access to properties which this contains, as the `HashMap`.
///
/// # Examples
///
/// ```rust
/// use sheave_core::{
///     ecma_array,
///     messages::amf::v0::{
///         EcmaArray,
///         Number
///     },
/// };
///
/// let ecma_array = ecma_array!(
///     "videocodecid" => Number::from(0)
/// );
/// ecma_array.get_properties().get("videocodecid");
/// &ecma_array.get_properties()["videocodecid"];
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EcmaArray(Properties);

impl EcmaArray {
    /// Constructs a new ECMA array.
    pub fn new(properties: Properties) -> Self {
        Self(properties)
    }

    /// Gets immutable properties from this array.
    pub fn get_properties(&self) -> &Properties {
        &self.0
    }

    /// Gets mutable properties from this array.
    pub fn get_properties_mut(&mut self) -> &mut Properties {
        &mut self.0
    }
}

impl Decoder<EcmaArray> for ByteBuffer {
    /// Decodes bytes into an ECMA array.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When buffer isn't remained at least 2 bytes. (non-empty ECMA array contains at least one pair of key and value)
    ///
    /// * [`InconsistentMarker`]
    ///
    /// When a marker byte doesn't indicate the ECMA array.
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
    ///         EcmaArray
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::EcmaArray as u8);
    /// buffer.put_u32_be(0);
    /// // Also ECMA array type is required a marker of object end (0x09) which is associated with an empty key.
    /// buffer.put_u16_be(0);
    /// buffer.put_u8(Marker::ObjectEnd as u8);
    /// assert!(Decoder::<EcmaArray>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Other as u8);
    /// buffer.put_u32_be(0);
    /// buffer.put_u16_be(0);
    /// buffer.put_u8(Marker::ObjectEnd as u8);
    /// assert!(Decoder::<EcmaArray>::decode(&mut buffer).is_err());
    ///
    /// // This is a missing sequence of the "sparkle heart(💖)".
    /// let mut bytes = vec![0, 159, 146, 150];
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::EcmaArray as u8);
    /// buffer.put_u32_be(0);
    /// buffer.put_u16_be(4);
    /// buffer.put_bytes(&bytes);
    /// buffer.put_u8(Marker::Number as u8);
    /// buffer.put_f64(0.0);
    /// buffer.put_u16_be(0);
    /// buffer.put_u8(Marker::ObjectEnd as u8);
    /// assert!(Decoder::<EcmaArray>::decode(&mut buffer).is_err());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<EcmaArray>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// Note the length field will not be so cared because to decode is enough to check the object end marker (0x09).
    /// However warning will emit if lengths is inconsistent.
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    fn decode(&mut self) -> IOResult<EcmaArray> {
        self.get_u8().and_then(
            |marker| ensure_marker(Marker::EcmaArray as u8, marker)
        )?;

        let length = self.get_u32_be()?;
        let properties: Properties = self.decode()?;

        if properties.len() != length as usize {
            warn!("Properties length doesn't match previous field: previous field: {length}, actual length: {}", properties.len());
        }

        Ok(EcmaArray(properties))
    }
}

impl Encoder<EcmaArray> for ByteBuffer {
    /// Encodes an ECMA array into bytes.
    ///
    /// # Panics
    ///
    /// Its length must be the range of 32 bits.
    /// If it exceeds, a panic is occured.
    fn encode(&mut self, ecma_array: &EcmaArray) {
        assert!(ecma_array.0.len() <= u32::MAX as usize);

        self.put_u8(Marker::EcmaArray as u8);
        self.put_u32_be(ecma_array.0.len() as u32);
        self.encode(&ecma_array.0);
    }
}

/// Constructs an ECMA array.
///
/// # Examples
///
/// ```rust
/// use sheave_core::{
///     ecma_array,
///     messages::amf::v0::{
///         EcmaArray,
///         Number
///     }
/// };
///
/// let mut on_metadata = EcmaArray::default();
/// on_metadata.get_properties_mut().insert("videocodecid", Number::from(0));
/// on_metadata.get_properties_mut().insert("audiocodecid", Number::from(0));
///
/// assert_eq!(
///     on_metadata,
///     ecma_array!(
///         "videocodecid" => Number::from(0),
///         "audiocodecid" => Number::from(0)
///     )
/// )
/// ```
#[macro_export]
macro_rules! ecma_array {
    ($($key:expr => $value:expr),*) => {
        {
            use $crate::messages::amf::v0::{
                EcmaArray,
                Properties
            };
            let mut properties = Properties::default();
            $(properties.insert($key, $value);)*
            EcmaArray::new(properties)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::amf::v0::UnmarkedString;
    use super::*;

    #[test]
    fn decode_ecma_array() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u8(Marker::EcmaArray as u8);
        buffer.put_u32_be(0);
        buffer.encode(&UnmarkedString::from(""));
        buffer.put_u8(Marker::ObjectEnd as u8);
        let result: IOResult<EcmaArray> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(EcmaArray::default(), actual)
    }

    #[test]
    fn encode_ecma_array() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&EcmaArray::default());
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::EcmaArray as u8, result[0]);
        assert_eq!(&0u32.to_be_bytes(), &result[1..5]);
        assert_eq!(&0u16.to_be_bytes(), &result[5..7]);
        assert_eq!(Marker::ObjectEnd as u8, result[7])
    }
}
