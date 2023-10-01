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
//! 1. `Marker` (1 byte)
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
//!
//! [`Marker`]: self::Marker
//! [`Number`]: self::Number
//! [`Boolean`]: self::Boolean
//! [`AmfString`]: self::AmfString

mod number;
mod boolean;
mod string;
mod object;

pub use self::{
    number::Number,
    boolean::Boolean,
    string::AmfString,
    object::Object
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
/// |`ObjectEnd`|`9`|
/// |`Other`|other numbers|
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    Number,
    Boolean,
    AmfString,
    Object,
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
