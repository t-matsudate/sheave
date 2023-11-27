mod invalid_protocol;

use std::{
    ffi::{
        OsStr,
        OsString
    },
    str::FromStr
};
use clap::{
    ValueEnum,
    builder::PossibleValue
};
pub use self::invalid_protocol::*;

/// The available protocol representation for CLI.
///
/// Currently, following protocols are available.
///
/// |Protocol|Value|
/// | :- | :- |
/// |RTMP|`Rtmp`|
#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    Rtmp
}

impl Protocol {
    const AVAILABLE_PROTOCOLS: &[Self] = &[
        Self::Rtmp
    ];
}

impl From<String> for Protocol {
    fn from(protocol: String) -> Self {
        use Protocol::*;
        let protocol = protocol.to_lowercase();
        if protocol == "rtmp" {
            Rtmp
        } else {
            unimplemented!("Protocol: {protocol} isn't avaialble. Currently following protocols are available: {:?}", Self::AVAILABLE_PROTOCOLS)
        }
    }
}

impl From<&str> for Protocol {
    fn from(protocol: &str) -> Self {
        Self::from(protocol.to_string())
    }
}

impl From<&OsStr> for Protocol {
    fn from(protocol: &OsStr) -> Self {
        let Some(protocol) = protocol.to_str() else {
            panic!("Protocol: Invalid for UTF-8: {protocol:?}")
        };
        Self::from(protocol)
    }
}

impl From<OsString> for Protocol {
    fn from(protocol: OsString) -> Self {
        Self::from(protocol.as_os_str())
    }
}

impl FromStr for Protocol {
    type Err = IOError;

    fn from_str(protocol_str: &str) -> Result<Self, Self::Err> {
        use Protocol::*;
        let protocol_str = protocol_str.to_lowercase();
        if protocol_str == "rtmp" {
            Ok(Rtmp)
        } else {
            Err(invalid_protocol(protocol_str))
        }
    }
}

impl ValueEnum for Protocol {
    fn value_variants<'a>() -> &'a [Self] {
        Self::AVAILABLE_PROTOCOLS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        use Protocol::*;
        Some(match self {
            Rtmp => PossibleValue::new("rtmp")
        })
    }
}
