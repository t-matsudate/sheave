use std::{
    future::Future,
    io::Result as IOResult,
    pin::Pin,
    task::{
        Context as FutureContext,
        Poll
    },
    time::Duration
};
use futures::ready;
use tokio::io::AsyncWrite;
use crate::messages::headers::{
    MessageHeader,
    New,
    SameSource,
    TimerChange
};

#[doc(hidden)]
#[derive(Debug)]
pub struct MessageHeaderWriter<'a, W: AsyncWrite> {
    writer: Pin<&'a mut W>,
    message_header: &'a MessageHeader
}

impl<W: AsyncWrite> MessageHeaderWriter<'_, W> {
    fn write_timestamp(&mut self, cx: &mut FutureContext<'_>, timestamp: Duration) -> Poll<IOResult<()>> {
        assert!(0x00ffffff >= timestamp.as_millis() as u32);
        self.writer.as_mut().poll_write(cx, &(timestamp.as_millis() as u32).to_be_bytes()[1..]).map_ok(|_| ())
    }

    fn write_message_length(&mut self, cx: &mut FutureContext<'_>, message_length: u32) -> Poll<IOResult<()>> {
        assert!(0x00ffffff >= message_length);
        self.writer.as_mut().poll_write(cx, &message_length.to_be_bytes()[1..]).map_ok(|_| ())
    }

    fn write_message_type(&mut self, cx: &mut FutureContext<'_>, message_type: u8) -> Poll<IOResult<()>> {
        self.writer.as_mut().poll_write(cx, &message_type.to_be_bytes()).map_ok(|_| ())
    }

    fn write_message_id(&mut self, cx: &mut FutureContext<'_>, message_id: u32) -> Poll<IOResult<()>> {
        self.writer.as_mut().poll_write(cx, &message_id.to_le_bytes()).map_ok(|_| ())
    }

    fn write_new(&mut self, cx: &mut FutureContext<'_>, new: &New) -> Poll<IOResult<()>> {
        ready!(self.write_timestamp(cx, new.timestamp))?;
        ready!(self.write_message_length(cx, new.message_length))?;
        ready!(self.write_message_type(cx, new.message_type))?;
        ready!(self.write_message_id(cx, new.message_id))?;
        Poll::Ready(Ok(()))
    }

    fn write_same_source(&mut self, cx: &mut FutureContext<'_>, same_source: &SameSource) -> Poll<IOResult<()>> {
        ready!(self.write_timestamp(cx, same_source.timestamp))?;
        ready!(self.write_message_length(cx, same_source.message_length))?;
        ready!(self.write_message_type(cx, same_source.message_type))?;
        Poll::Ready(Ok(()))
    }

    fn write_timer_change(&mut self, cx: &mut FutureContext<'_>, timer_change: &TimerChange) -> Poll<IOResult<()>> {
        ready!(self.write_timestamp(cx, timer_change.timestamp))?;
        Poll::Ready(Ok(()))
    }
}

#[doc(hidden)]
impl<W: AsyncWrite> Future for MessageHeaderWriter<'_, W> {
    type Output = IOResult<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        match self.message_header {
            &MessageHeader::New(ref new) => self.write_new(cx, new),
            &MessageHeader::SameSource(ref same_source) => self.write_same_source(cx, same_source),
            &MessageHeader::TimerChange(ref timer_change) => self.write_timer_change(cx, timer_change),
            _ => Poll::Ready(Ok(()))
        }
    }
}

/// Writes message header from stream.
///
/// # Panics
///
/// In the specification, timestamps and message lengths are defined as 3 bytes, therefore any value above `0x00ffffff` is emitted an assertion error.
///
/// # Examples
///
/// ```rust
/// use std::{
///     cmp::min,
///     io::Result as IOResult,
///     pin::{
///         Pin,
///         pin
///     },
///     time::Duration
/// };
/// use rand::random;
/// use sheave_core::{
///     messages::headers::{
///         MessageHeader,
///         New,
///         SameSource,
///         TimerChange
///     },
///     writers::write_message_header
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     // In case of 11 bytes.
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let message_header = MessageHeader::New(
///         New {
///             timestamp: Duration::from_millis(min(0x00ffffff, random::<u32>()) as u64),
///             message_length: min(0x00ffffff, random::<u32>()),
///             message_type: random::<u8>(),
///             message_id: random::<u32>()
///         }
///     );
///     write_message_header(writer.as_mut(), &message_header).await?;
///     let mut written: [u8; 4] = [0; 4];
///     written[1..].copy_from_slice(&writer[..3]);
///     let timestamp = Duration::from_millis(u32::from_be_bytes(written) as u64);
///     assert_eq!(timestamp, message_header.get_timestamp().unwrap());
///     let mut written: [u8; 4] = [0; 4];
///     written[1..].copy_from_slice(&writer[3..6]);
///     let message_length = u32::from_be_bytes(written);
///     assert_eq!(message_length, message_header.get_message_length().unwrap());
///     let message_type = writer[6];
///     assert_eq!(message_type, message_header.get_message_type().unwrap());
///     let mut written: [u8; 4] = [0; 4];
///     written.copy_from_slice(&writer[7..]);
///     let message_id = u32::from_le_bytes(written);
///     assert_eq!(message_id, message_header.get_message_id().unwrap());
///
///     // In case of 7 bytes.
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let message_header = MessageHeader::SameSource(
///         SameSource {
///             timestamp: Duration::from_millis(min(0x00ffffff, random::<u32>()) as u64),
///             message_length: min(0x00ffffff, random::<u32>()),
///             message_type: random::<u8>()
///         }
///     );
///     write_message_header(writer.as_mut(), &message_header).await?;
///     let mut written: [u8; 4] = [0; 4];
///     written[1..].copy_from_slice(&writer[..3]);
///     let timestamp = Duration::from_millis(u32::from_be_bytes(written) as u64);
///     assert_eq!(timestamp, message_header.get_timestamp().unwrap());
///     let mut written: [u8; 4] = [0; 4];
///     written[1..].copy_from_slice(&writer[3..6]);
///     let message_length = u32::from_be_bytes(written);
///     assert_eq!(message_length, message_header.get_message_length().unwrap());
///     let message_type = writer[6];
///     assert_eq!(message_type, message_header.get_message_type().unwrap());
///
///     // In case of 3 bytes.
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let message_header = MessageHeader::TimerChange(
///         TimerChange {
///             timestamp: Duration::from_millis(min(0x00ffffff, random::<u32>()) as u64)
///         }
///     );
///     write_message_header(writer.as_mut(), &message_header).await?;
///     let mut written: [u8; 4] = [0; 4];
///     written[1..].copy_from_slice(&writer[..3]);
///     let timestamp = Duration::from_millis(u32::from_be_bytes(written) as u64);
///     assert_eq!(timestamp, message_header.get_timestamp().unwrap());
///
///     // In case of 0 bytes. (Continue)
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let message_header = MessageHeader::Continue;
///     write_message_header(writer.as_mut(), &message_header).await?;
///     assert!(writer.is_empty());
///
///     Ok(())
/// }
/// ```
pub fn write_message_header<'a, W: AsyncWrite>(writer: Pin<&'a mut W>, message_header: &'a MessageHeader) -> MessageHeaderWriter<'a, W> {
    MessageHeaderWriter { writer, message_header }
}

#[cfg(test)]
mod tests {
    use std::{
        cmp::min,
        pin::pin
    };
    use rand::random;
    use super::*;

    #[tokio::test]
    async fn write_new() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let message_header = MessageHeader::New(
            New {
                timestamp: Duration::from_millis(min(0x00ffffff, random::<u32>()) as u64),
                message_length: min(0x00ffffff, random::<u32>()),
                message_type: random::<u8>(),
                message_id: random::<u32>()
            }
        );
        let result = write_message_header(writer.as_mut(), &message_header).await;
        assert!(result.is_ok());
        let mut written: [u8; 4] = [0; 4];
        written[1..].copy_from_slice(&writer[..3]);
        let timestamp = Duration::from_millis(u32::from_be_bytes(written) as u64);
        assert_eq!(timestamp, message_header.get_timestamp().unwrap());
        let mut written: [u8; 4] = [0; 4];
        written[1..].copy_from_slice(&writer[3..6]);
        let message_length = u32::from_be_bytes(written);
        assert_eq!(message_length, message_header.get_message_length().unwrap());
        let message_type = writer[6];
        assert_eq!(message_type, message_header.get_message_type().unwrap());
        let mut written: [u8; 4] = [0; 4];
        written.copy_from_slice(&writer[7..]);
        let message_id = u32::from_le_bytes(written);
        assert_eq!(message_id, message_header.get_message_id().unwrap())
    }

    #[tokio::test]
    async fn write_same_source() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let message_header = MessageHeader::SameSource(
            SameSource {
                timestamp: Duration::from_millis(min(0x00ffffff, random::<u32>()) as u64),
                message_length: min(0x00ffffff, random::<u32>()),
                message_type: random::<u8>()
            }
        );
        let result = write_message_header(writer.as_mut(), &message_header).await;
        assert!(result.is_ok());
        let mut written: [u8; 4] = [0; 4];
        written[1..].copy_from_slice(&writer[..3]);
        let timestamp = Duration::from_millis(u32::from_be_bytes(written) as u64);
        assert_eq!(timestamp, message_header.get_timestamp().unwrap());
        let mut written: [u8; 4] = [0; 4];
        written[1..].copy_from_slice(&writer[3..6]);
        let message_length = u32::from_be_bytes(written);
        assert_eq!(message_length, message_header.get_message_length().unwrap());
        let message_type = writer[6];
        assert_eq!(message_type, message_header.get_message_type().unwrap())
    }

    #[tokio::test]
    async fn write_timer_change() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let message_header = MessageHeader::TimerChange(
            TimerChange {
                timestamp: Duration::from_millis(min(0x00ffffff, random::<u32>()) as u64)
            }
        );
        let result = write_message_header(writer.as_mut(), &message_header).await;
        assert!(result.is_ok());
        let mut written: [u8; 4] = [0; 4];
        written[1..].copy_from_slice(&writer[..3]);
        let timestamp = Duration::from_millis(u32::from_be_bytes(written) as u64);
        assert_eq!(timestamp, message_header.get_timestamp().unwrap())
    }

    #[tokio::test]
    async fn write_continue() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let message_header = MessageHeader::Continue;
        write_message_header(writer.as_mut(), &message_header).await.unwrap();
        assert!(writer.is_empty())
    }
}
