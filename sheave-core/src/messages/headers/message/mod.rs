mod new;
mod same_source;
mod timer_change;

use std::time::Duration;
pub use self::{
    new::New,
    same_source::SameSource,
    timer_change::TimerChange
};
use MessageHeader::*;

#[derive(Debug, Clone, Copy)]
pub enum MessageHeader {
    New(New),
    SameSource(SameSource),
    TimerChange(TimerChange),
    Continue
}

impl MessageHeader {
    pub fn get_timestamp(&self) -> Option<Duration> {
        match *self {
            New(new) => Some(new.timestamp),
            SameSource(same_source) => Some(same_source.timestamp),
            TimerChange(timer_change) => Some(timer_change.timestamp),
            _ => None
        }
    }

    pub fn get_message_length(&self) -> Option<u32> {
        match *self {
            New(new) => Some(new.message_length),
            SameSource(same_source) => Some(same_source.message_length),
            _ => None
        }
    }

    pub fn get_message_type(&self) -> Option<u8> {
        match *self {
            New(new) => Some(new.message_type),
            SameSource(same_source) => Some(same_source.message_type),
            _ => None
        }
    }

    pub fn get_message_id(&self) -> Option<u32> {
        match *self {
            New(new) => Some(new.message_id),
            _ => None
        }
    }
}
