use std::io::Result as IOResult;
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
///
/// You can access to properties which this constains, as the `HashMap`.
///
/// # Example
///
/// ```rust
/// use sheave_core::{
///     messages::amf::v0::AmfString,
///     object
/// };
///
/// let mut object = object!(
///     "app" => AmfString::from("ondemand")
/// );
/// object.get_properties().get("app");
/// &object.get_properties()["app"];
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Object(Properties);

impl Object {
    /// Constrcuts a new object.
    pub fn new(properties: Properties) -> Self {
        Self(properties)
    }

    /// Gets immutable properties from this object.
    pub fn get_properties(&self) -> &Properties {
        &self.0
    }

    /// Gets mutable properties from this object.
    pub fn get_properties_mut(&mut self) -> &mut Properties {
        &mut self.0
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
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    fn decode(&mut self) -> IOResult<Object> {
        self.get_u8().and_then(
            |marker| ensure_marker(Marker::Object as u8, marker)
        )?;

        let properties: Properties = self.decode()?;
        Ok(Object(properties))
    }
}

impl Encoder<Object> for ByteBuffer {
    /// Encodes an AMF's Object into bytes.
    fn encode(&mut self, object: &Object) {
        self.put_u8(Marker::Object as u8);
        self.encode(&object.0);
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
///         AmfString,
///         Object
///     }
/// };
///
/// let mut command_object = Object::default();
/// command_object.get_properties_mut().insert("app", AmfString::from("ondemand"));
/// command_object.get_properties_mut().insert("type", AmfString::from("nonprivate"));
/// command_object.get_properties_mut().insert("flashVer", AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"));
/// command_object.get_properties_mut().insert("tcUrl", AmfString::from("rtmp://localhost"));
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
            use $crate::messages::amf::v0::{
                Object,
                Properties
            };

            let mut properties = Properties::default();
            $(properties.insert($key, $value);)*
            Object::new(properties)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::amf::v0::UnmarkedString;
    use super::*;

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
