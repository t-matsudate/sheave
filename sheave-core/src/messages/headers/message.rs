mod message_type;
mod new;
mod same_source;
mod timer_change;

use std::time::Duration;
pub use self::{
    message_type::MessageType,
    new::New,
    same_source::SameSource,
    timer_change::TimerChange
};
use MessageHeader::*;

/// Indicates a chunk datum format and which stream is it into.
/// This header has 4 types.
///
/// |Total Length|Timestamp|Message Length|Message Type|Message ID|
/// | ---------: | ------: | -----------: | ---------: | -------: |
/// |11          |3        |3             |1           |4         |
/// |7           |3        |3             |1           |          |
/// |3           |3        |              |            |          |
/// |0           |         |              |            |          |
///
/// Unit of every item is bytes.
///
/// * 11 bytes type is required when a new message ID is necessary, that is, communicating with a partner on a new message stream.
/// * 7 bytes type is required when a message which is different either type or length, sends/receives on same message stream.
/// * 3 bytes type is required when a message which is same both type and length, sends/receives on same message stream.
/// * 0 bytes type is required when concatenates a message which is same but exceeding the chunk size on same message stream.
///
/// Note that 0 bytes type is required to consider of the message length becuase is inside its chunk datum.
///
/// Headers and tuples are convertible into each other.
///
/// |Headers                     |Tuples                    |
/// | :------------------------- | :----------------------- |
/// |`MessageHeader::New`        |`(Duration, u32, u8, u32)`|
/// |`MessageHeader::SameSource` |`(Duration, u32, u8)`     |
/// |`MessageHeader::TimerChange`|`Duration`                |
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use sheave_core::messages::headers::MessageHeader;
///
/// let new = MessageHeader::New((Duration::default(), u32::default(), u8::default().into(), u32::default()).into());
/// assert!(new.get_message_id().is_some());
///
/// let same_source = MessageHeader::SameSource((Duration::default(), u32::default(), u8::default().into()).into());
/// assert!(same_source.get_message_id().is_none());
///
/// let timer_change = MessageHeader::TimerChange(Duration::default().into());
/// assert!(timer_change.get_message_length().is_none());
/// assert!(timer_change.get_message_type().is_none());
/// ```
///
/// Because of no field, The Continue header isn't implemented `From<T>`.
#[derive(Debug, Clone, Copy)]
pub enum MessageHeader {
    New(New),
    SameSource(SameSource),
    TimerChange(TimerChange),
    Continue
}

impl MessageHeader {
    pub const U24_MAX: u32 = 0x00ffffff;

    /// Gets a timestamp.
    /// Only 0 bytes type returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::messages::headers::MessageHeader;
    ///
    /// // In case of 11 bytes type.
    /// let new = MessageHeader::New((Duration::default(), u32::default(), u8::default().into(), u32::default()).into());
    /// assert!(new.get_timestamp().is_some());
    ///
    /// // In case of 7 bytes type.
    /// let same_source = MessageHeader::SameSource((Duration::default(), u32::default(), u8::default().into()).into());
    /// assert!(same_source.get_timestamp().is_some());
    ///
    /// // In case of 3 bytes type.
    /// let timer_change = MessageHeader::TimerChange(Duration::default().into());
    /// assert!(timer_change.get_timestamp().is_some());
    /// 
    /// // In case of 0 bytes type.
    /// assert!(MessageHeader::Continue.get_timestamp().is_none())
    /// ```
    pub fn get_timestamp(&self) -> Option<Duration> {
        match *self {
            New(new) => Some(new.timestamp),
            SameSource(same_source) => Some(same_source.timestamp),
            TimerChange(timer_change) => Some(timer_change.timestamp),
            _ => None
        }
    }

    /// Gets a message length.
    /// 0 bytes type and 3 bytes type return `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::messages::headers::MessageHeader;
    ///
    /// // In case of 11 bytes type.
    /// let new = MessageHeader::New((Duration::default(), u32::default(), u8::default().into(), u32::default()).into());
    /// assert!(new.get_message_length().is_some());
    ///
    /// // In case of 7 bytes type.
    /// let same_source = MessageHeader::SameSource((Duration::default(), u32::default(), u8::default().into()).into());
    /// assert!(same_source.get_message_length().is_some());
    ///
    /// // In case of 3 bytes type.
    /// let timer_change = MessageHeader::TimerChange(Duration::default().into());
    /// assert!(timer_change.get_message_length().is_none());
    ///
    /// // In case of 0 bytes type.
    /// assert!(MessageHeader::Continue.get_message_length().is_none())
    /// ```
    pub fn get_message_length(&self) -> Option<u32> {
        match *self {
            New(new) => Some(new.message_length),
            SameSource(same_source) => Some(same_source.message_length),
            _ => None
        }
    }

    /// Gets a message type.
    /// 0 bytes type and 3 bytes type return `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::messages::headers::MessageHeader;
    ///
    /// // In case of 11 bytes type.
    /// let new = MessageHeader::New((Duration::default(), u32::default(), u8::default().into(), u32::default()).into());
    /// assert!(new.get_message_type().is_some());
    ///
    /// // In case of 7 bytes type.
    /// let same_source = MessageHeader::SameSource((Duration::default(), u32::default(), u8::default().into()).into());
    /// assert!(same_source.get_message_type().is_some());
    ///
    /// // In case of 3 bytes type.
    /// let timer_change = MessageHeader::TimerChange(Duration::default().into());
    /// assert!(timer_change.get_message_type().is_none());
    ///
    /// // In case of 0 bytes type.
    /// assert!(MessageHeader::Continue.get_message_type().is_none())
    /// ```
    pub fn get_message_type(&self) -> Option<MessageType> {
        match *self {
            New(new) => Some(new.message_type),
            SameSource(same_source) => Some(same_source.message_type),
            _ => None
        }
    }

    /// Gets a message ID.
    /// All but 11 bytes type returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::messages::headers::MessageHeader;
    ///
    /// // In case of 11 bytes type.
    /// let new = MessageHeader::New((Duration::default(), u32::default(), u8::default().into(), u32::default()).into());
    /// assert!(new.get_message_id().is_some());
    ///
    /// // In case of 7 bytes type.
    /// let same_source = MessageHeader::SameSource((Duration::default(), u32::default(), u8::default().into()).into());
    /// assert!(same_source.get_message_id().is_none());
    ///
    /// // In case of 3 bytes type.
    /// let timer_change = MessageHeader::TimerChange(Duration::default().into());
    /// assert!(timer_change.get_message_id().is_none());
    ///
    /// // In case of 0 bytes type.
    /// assert!(MessageHeader::Continue.get_message_id().is_none())
    /// ```
    pub fn get_message_id(&self) -> Option<u32> {
        match *self {
            New(new) => Some(new.message_id),
            _ => None
        }
    }
}
