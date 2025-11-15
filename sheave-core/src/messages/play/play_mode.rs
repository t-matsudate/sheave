use std::cmp::Ordering;
use crate::messages::amf::v0::Number;

/// Representation of the way to subscribe its stream.
///
/// Variants correspond to respectively following numbers:
///
/// |Pattern|Number|
/// | :- | :- |
/// |`Other`|below than `-2`|
/// |`Both`|`-2`|
/// |`Live`|`-1`|
/// |`Recorded`|above `0`|
///
/// Note this constins signed numbers above -2.
#[repr(i64)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayMode {
    Other = i64::MIN,
    #[default]
    Both = -2,
    Live,
    Record
}

impl From<i64> for PlayMode {
    fn from(play_mode: i64) -> Self {
        use PlayMode::*;

        match play_mode {
            -2 => Both,
            -1 => Live,
            play_mode if play_mode >= 0 => Record,
            _ => Other
        }
    }
}

impl From<PlayMode> for i64 {
    fn from(play_mode: PlayMode) -> Self {
        play_mode as i64
    }
}

impl From<Number> for PlayMode {
    fn from(play_mode: Number) -> Self {
        Self::from(play_mode.as_signed_integer())
    }
}

impl From<PlayMode> for Number {
    fn from(play_mode: PlayMode) -> Self {
        Self::new(play_mode as i64 as f64)
    }
}

impl PartialEq<i64> for PlayMode {
    fn eq(&self, other: &i64) -> bool {
        i64::from(*self).eq(other)
    }
}

impl PartialOrd<i64> for PlayMode {
    fn partial_cmp(&self, other: &i64) -> Option<Ordering> {
        i64::from(*self).partial_cmp(other)
    }
}

impl PartialEq<PlayMode> for i64 {
    fn eq(&self, other: &PlayMode) -> bool {
        self.eq(&(*other as i64))
    }
}

impl PartialOrd<PlayMode> for i64 {
    fn partial_cmp(&self, other: &PlayMode) -> Option<Ordering> {
        self.partial_cmp(&(*other as i64))
    }
}
