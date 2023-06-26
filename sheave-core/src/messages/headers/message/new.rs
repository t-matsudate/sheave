use std::time::Duration;

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct New {
    pub(super) timestamp: Duration,
    pub(super) message_length: u32,
    pub(super) message_type: u8,
    pub(super) message_id: u32
}

#[doc(hidden)]
impl From<(Duration, u32, u8, u32)> for New {
    fn from(new: (Duration, u32, u8, u32)) -> Self {
        Self {
            timestamp: new.0,
            message_length: new.1,
            message_type: new.2,
            message_id: new.3
        }
    }
}

#[doc(hidden)]
impl From<New> for (Duration, u32, u8, u32) {
    fn from(new: New) -> Self {
        (new.timestamp, new.message_length, new.message_type, new.message_id)
    }
}
