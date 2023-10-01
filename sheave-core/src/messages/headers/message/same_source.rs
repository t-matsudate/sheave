use std::time::Duration;
use super::MessageType;

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct SameSource {
    pub(super) timestamp: Duration,
    pub(super) message_length: u32,
    pub(super) message_type: MessageType
}

#[doc(hidden)]
impl From<(Duration, u32, MessageType)> for SameSource {
    fn from(same_source: (Duration, u32, MessageType)) -> Self {
        Self {
            timestamp: same_source.0,
            message_length: same_source.1,
            message_type: same_source.2
        }
    }
}

#[doc(hidden)]
impl From<SameSource> for (Duration, u32, MessageType) {
    fn from(same_source: SameSource) -> Self {
        (same_source.timestamp, same_source.message_length, same_source.message_type)
    }
}
