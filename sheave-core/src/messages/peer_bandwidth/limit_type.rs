/// Representation of the limit type field in the PeerBandwidth message.
///
/// Variants correspond to respectively following numbers:
///
/// |Pattern|Number|
/// | :- | :- |
/// |`Hard`|`0`|
/// |`Soft`|`1`|
/// |`Dynamic`|`2`|
/// |`Other`|other numbers|
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum LimitType {
    Hard,
    Soft,
    #[default]
    Dynamic,
    Other = 0xff
}

impl From<u8> for LimitType {
    fn from(limit_type: u8) -> Self {
        use LimitType::*;

        match limit_type {
            0 => Hard,
            1 => Soft,
            2 => Dynamic,
            _ => Other
        }
    }
}

impl From<LimitType> for u8 {
    fn from(limit_type: LimitType) -> Self {
        limit_type as u8
    }
}
