/// The first 2 bits to indicate a format of message header.
///
/// Variants correspond to respectively formats:
///
/// |Pattern|Format(length)|
/// | :- | -: |
/// |`New`|11 bytes|
/// |`SameSource`|7 bytes|
/// |`TimerChange`|3 bytes|
/// |`Continue`|0 bytes|
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageFormat {
    New,
    SameSource,
    TimerChange,
    Continue
}

impl From<u8> for MessageFormat {
    /// Converts message format bits into a variant.
    ///
    /// # Panics
    ///
    /// Because of the RTMP specification, this is implemented in such a way as to emit a panic when is passed any value above 3.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::panic::catch_unwind;
    /// use sheave_core::messages::headers::{
    ///     MessageFormat,
    ///     MessageFormat::*
    /// };
    ///
    /// assert_eq!(New, MessageFormat::from(0)); // => ok
    /// assert_eq!(SameSource, MessageFormat::from(1)); // => ok
    /// assert_eq!(TimerChange, MessageFormat::from(2)); // => ok
    /// assert_eq!(Continue, MessageFormat::from(3)); // => ok
    /// assert!(catch_unwind(|| MessageFormat::from(4)).is_err()); // => this will be backtrace.
    /// ```
    fn from(message_format: u8) -> Self {
        use MessageFormat::*;

        match message_format {
            0 => New,
            1 => SameSource,
            2 => TimerChange,
            3 => Continue,
            _ => unreachable!("MessageFormat.")
        }
    }
}

impl From<MessageFormat> for u8 {
    fn from(message_format: MessageFormat) -> Self {
        message_format as u8
    }
}
