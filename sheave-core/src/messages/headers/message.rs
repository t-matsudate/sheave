mod message_type;

use std::time::Duration;
pub use self::message_type::*;

/// Indicates a chunk datum format and which stream is it into.
///
/// This header has 4 types.
///
/// |Total|Timestamp|Message Length|Message Type|Message ID|
/// | -: | -: | -: | -: | -: |
/// |11|3|3|1|4|
/// |7 |3|3|1| |
/// |3 |3| | | |
/// |0 | | | | |
///
/// The unit of every length item is bytes.
///
/// * 11 bytes type is required when a new message ID is necessary, that is, communicating with a partner on a new message stream.
/// * 7 bytes type is required when a message which is different either type or length, sends/receives on same message stream.
/// * 3 bytes type is required when a message which is same both type and length, sends/receives on same message stream.
/// * 0 bytes type is required when concatenates a message which is same both type and length, but exceeding the chunk size on same message stream.
///
/// Note that 0 bytes type is required to consider of the message length because can be contained into its chunk datum.
///
/// tuples are convertible into message headers.
///
/// |Tuples|Message Header Format|
/// | :- | :- |
/// |`(Duration, u32, u8, u32)`|`New`|
/// |`(Duration, u32, u8)|`SameSource`|
/// |`Duration`|`TimerChange`|
/// |`()`|`Continue`|
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use sheave_core::messages::headers::MessageHeader;
///
/// let new: MessageHeader = (Duration::default(), u32::default(), u8::default().into(), u32::default()).into();
/// assert!(new.get_message_id().is_some());
///
/// let same_source: MessageHeader = (Duration::default(), u32::default(), u8::default().into()).into();
/// assert!(same_source.get_message_length().is_some());
/// assert!(same_source.get_message_type().is_some());
///
/// let timer_change: MessageHeader = Duration::default().into();
/// assert!(timer_change.get_timestamp().is_some());
///
/// let cont: MessageHeader = ().into();
/// assert!(cont.get_timestamp().is_none())
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MessageHeader {
    timestamp: Option<Duration>,
    message_length: Option<u32>,
    message_type: Option<MessageType>,
    message_id: Option<u32>
}

impl MessageHeader {
    /// Gets a timestamp.
    ///
    /// Only 0 bytes type returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::messages::headers::MessageHeader;
    ///
    /// // In case of 11 bytes type.
    /// let new: MessageHeader = (Duration::default(), u32::default(), u8::default().into(), u32::default()).into();
    /// assert!(new.get_timestamp().is_some());
    ///
    /// // In case of 7 bytes type.
    /// let same_source: MessageHeader = (Duration::default(), u32::default(), u8::default().into()).into();
    /// assert!(same_source.get_timestamp().is_some());
    ///
    /// // In case of 3 bytes type.
    /// let timer_change: MessageHeader = Duration::default().into();
    /// assert!(timer_change.get_timestamp().is_some());
    /// 
    /// // In case of 0 bytes type.
    /// let cont: MessageHeader = ().into();
    /// assert!(cont.get_timestamp().is_none())
    /// ```
    pub fn get_timestamp(&self) -> Option<Duration> {
        self.timestamp
    }

    /// Gets a message length.
    ///
    /// 0 bytes type and 3 bytes type return `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::messages::headers::MessageHeader;
    ///
    /// // In case of 11 bytes type.
    /// let new: MessageHeader = (Duration::default(), u32::default(), u8::default().into(), u32::default()).into();
    /// assert!(new.get_message_length().is_some());
    ///
    /// // In case of 7 bytes type.
    /// let same_source: MessageHeader = (Duration::default(), u32::default(), u8::default().into()).into();
    /// assert!(same_source.get_message_length().is_some());
    ///
    /// // In case of 3 bytes type.
    /// let timer_change: MessageHeader = Duration::default().into();
    /// assert!(timer_change.get_message_length().is_none());
    ///
    /// // In case of 0 bytes type.
    /// let cont: MessageHeader = ().into();
    /// assert!(cont.get_message_length().is_none())
    /// ```
    pub fn get_message_length(&self) -> Option<u32> {
        self.message_length
    }

    /// Gets a message type.
    ///
    /// 0 bytes type and 3 bytes type return `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::messages::headers::MessageHeader;
    ///
    /// // In case of 11 bytes type.
    /// let new: MessageHeader = (Duration::default(), u32::default(), u8::default().into(), u32::default()).into();
    /// assert!(new.get_message_type().is_some());
    ///
    /// // In case of 7 bytes type.
    /// let same_source: MessageHeader = (Duration::default(), u32::default(), u8::default().into()).into();
    /// assert!(same_source.get_message_type().is_some());
    ///
    /// // In case of 3 bytes type.
    /// let timer_change: MessageHeader = Duration::default().into();
    /// assert!(timer_change.get_message_type().is_none());
    ///
    /// // In case of 0 bytes type.
    /// let cont: MessageHeader = ().into();
    /// assert!(cont.get_message_type().is_none())
    /// ```
    pub fn get_message_type(&self) -> Option<MessageType> {
        self.message_type
    }

    /// Gets a message ID.
    ///
    /// All but 11 bytes type returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::messages::headers::MessageHeader;
    ///
    /// // In case of 11 bytes type.
    /// let new: MessageHeader = (Duration::default(), u32::default(), u8::default().into(), u32::default()).into();
    /// assert!(new.get_message_id().is_some());
    ///
    /// // In case of 7 bytes type.
    /// let same_source: MessageHeader = (Duration::default(), u32::default(), u8::default().into()).into();
    /// assert!(same_source.get_message_id().is_none());
    ///
    /// // In case of 3 bytes type.
    /// let timer_change: MessageHeader = Duration::default().into();
    /// assert!(timer_change.get_message_id().is_none());
    ///
    /// // In case of 0 bytes type.
    /// let cont: MessageHeader = ().into();
    /// assert!(cont.get_message_id().is_none())
    /// ```
    pub fn get_message_id(&self) -> Option<u32> {
        self.message_id
    }
}

impl From<(Duration, u32, MessageType, u32)> for MessageHeader {
    fn from((timestamp, message_length, message_type, message_id): (Duration, u32, MessageType, u32)) -> Self {
        Self {
            timestamp: Some(timestamp),
            message_length: Some(message_length),
            message_type: Some(message_type),
            message_id: Some(message_id)
        }
    }
}

impl From<(Duration, u32, MessageType)> for MessageHeader {
    fn from((timestamp, message_length, message_type): (Duration, u32, MessageType)) -> Self {
        Self {
            timestamp: Some(timestamp),
            message_length: Some(message_length),
            message_type: Some(message_type),
            ..Self::default()
        }
    }
}

impl From<Duration> for MessageHeader {
    fn from(timestamp: Duration) -> Self {
        Self {
            timestamp: Some(timestamp),
            ..Self::default()
        }
    }
}

impl From<()> for MessageHeader {
    fn from(_fields: ()) -> Self {
        Self::default()
    }
}

impl From<MessageHeader> for (Option<Duration>, Option<u32>, Option<MessageType>, Option<u32>) {
    fn from(message_header: MessageHeader) -> Self {
        (
            message_header.get_timestamp(),
            message_header.get_message_length(),
            message_header.get_message_type(),
            message_header.get_message_id()
        )
    }
}
