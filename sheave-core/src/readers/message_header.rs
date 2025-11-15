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
use tokio::io::{
    AsyncRead,
    ReadBuf
};
use crate::messages::headers::{
    MessageHeader,
    MessageFormat,
    MessageType
};

#[doc(hidden)]
#[derive(Debug)]
pub struct MessageHeaderReader<'a, R: AsyncRead> {
    reader: Pin<&'a mut R>,
    message_format: MessageFormat
}

#[doc(hidden)]
impl<R: AsyncRead> MessageHeaderReader<'_, R> {
    fn read_timestamp(&mut self, cx: &mut FutureContext<'_>) -> Poll<IOResult<Duration>> {
        let mut timestamp_bytes: [u8; 4] = [0; 4];
        let mut buf = ReadBuf::new(&mut timestamp_bytes[1..]);
        ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
        Poll::Ready(Ok(Duration::from_millis(u32::from_be_bytes(timestamp_bytes) as u64)))
    }

    fn read_message_length(&mut self, cx: &mut FutureContext<'_>) -> Poll<IOResult<u32>> {
        let mut message_length_bytes: [u8; 4] = [0; 4];
        let mut buf = ReadBuf::new(&mut message_length_bytes[1..]);
        ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
        Poll::Ready(Ok(u32::from_be_bytes(message_length_bytes)))
    }

    fn read_message_type(&mut self, cx: &mut FutureContext<'_>) -> Poll<IOResult<MessageType>> {
        let mut message_type_byte: [u8; 1] = [0; 1];
        let mut buf = ReadBuf::new(&mut message_type_byte);
        ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
        Poll::Ready(Ok(u8::from_be_bytes(message_type_byte).into()))
    }

    fn read_message_id(&mut self, cx: &mut FutureContext<'_>) -> Poll<IOResult<u32>> {
        let mut message_id_bytes: [u8; 4] = [0; 4];
        let mut buf = ReadBuf::new(&mut message_id_bytes);
        ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
        Poll::Ready(Ok(u32::from_le_bytes(message_id_bytes)))
    }

    fn read_new(&mut self, cx: &mut FutureContext<'_>) -> Poll<IOResult<MessageHeader>> {
        let timestamp = ready!(self.read_timestamp(cx))?;
        let message_length = ready!(self.read_message_length(cx))?;
        let message_type = ready!(self.read_message_type(cx))?;
        let message_id = ready!(self.read_message_id(cx))?;
        Poll::Ready(Ok((timestamp, message_length, message_type, message_id).into()))
    }

    fn read_same_source(&mut self, cx: &mut FutureContext<'_>) -> Poll<IOResult<MessageHeader>> {
        let timestamp = ready!(self.read_timestamp(cx))?;
        let message_length = ready!(self.read_message_length(cx))?;
        let message_type = ready!(self.read_message_type(cx))?;
        Poll::Ready(Ok((timestamp, message_length, message_type).into()))
    }

    fn read_timer_change(&mut self, cx: &mut FutureContext<'_>) -> Poll<IOResult<MessageHeader>> {
        let timestamp = ready!(self.read_timestamp(cx))?;
        Poll::Ready(Ok(timestamp.into()))
    }
}

#[doc(hidden)]
impl<R: AsyncRead> Future for MessageHeaderReader<'_, R> {
    type Output = IOResult<MessageHeader>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        match self.message_format {
            MessageFormat::New => self.read_new(cx),
            MessageFormat::SameSource => self.read_same_source(cx),
            MessageFormat::TimerChange => self.read_timer_change(cx),
            MessageFormat::Continue => Poll::Ready(Ok(MessageHeader::Continue))
        }
    }
}

/// Reads a message header from streams.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::pin,
///     time::Duration
/// };
/// use rand::random;
/// use sheave_core::{
///     messages::headers::{
///         MessageHeader,
///         MessageFormat::*,
///         MessageType
///     },
///     readers::read_message_header
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     // In case of 11 bytes.
///     let mut reader: [u8; 11] = [0; 11];
///     let timestamp = random::<u32>() << 8 >> 8;
///     let message_length = random::<u32>() << 8 >> 8;
///     let message_type = random::<u8>();
///     let message_id = random::<u32>();
///     reader[..3].copy_from_slice(&timestamp.to_be_bytes()[1..]);
///     reader[3..6].copy_from_slice(&message_length.to_be_bytes()[1..]);
///     reader[6] = message_type;
///     reader[7..].copy_from_slice(&message_id.to_le_bytes());
///     let result = read_message_header(pin!(reader.as_slice()), New).await?;
///     assert_eq!(Duration::from_millis(timestamp as u64), result.get_timestamp().unwrap());
///     assert_eq!(message_length, result.get_message_length().unwrap());
///     assert_eq!(MessageType::from(message_type), result.get_message_type().unwrap());
///     assert_eq!(message_id, result.get_message_id().unwrap());
///
///     // In case of 7 bytes.
///     let mut reader: [u8; 7] = [0; 7];
///     let timestamp = random::<u32>() << 8 >> 8;
///     let message_length = random::<u32>() << 8 >> 8;
///     let message_type = random::<u8>();
///     reader[..3].copy_from_slice(&timestamp.to_be_bytes()[1..]);
///     reader[3..6].copy_from_slice(&message_length.to_be_bytes()[1..]);
///     reader[6] = message_type;
///     let result = read_message_header(pin!(reader.as_slice()), SameSource).await?;
///     assert_eq!(Duration::from_millis(timestamp as u64), result.get_timestamp().unwrap());
///     assert_eq!(message_length, result.get_message_length().unwrap());
///     assert_eq!(MessageType::from(message_type), result.get_message_type().unwrap());
///
///     // In case of 3 bytes.
///     let mut reader: [u8; 3] = [0; 3];
///     let timestamp = random::<u32>() << 8 >> 8;
///     reader.copy_from_slice(&timestamp.to_be_bytes()[1..]);
///     let result = read_message_header(pin!(reader.as_slice()), TimerChange).await?;
///     assert_eq!(Duration::from_millis(timestamp as u64), result.get_timestamp().unwrap());
///
///     // In case of 0 bytes. (Continue)
///     let mut reader: [u8; 0] = [0; 0];
///     let result = read_message_header(pin!(reader.as_slice()), Continue).await?;
///     assert!(result.get_timestamp().is_none());
///     Ok(())
/// }
/// ```
pub fn read_message_header<R: AsyncRead>(reader: Pin<&mut R>, message_format: MessageFormat) -> MessageHeaderReader<'_, R> {
    MessageHeaderReader { reader, message_format }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use rand::random;
    use crate::messages::headers::MessageFormat::*;
    use super::*;

    #[tokio::test]
    async fn read_new() {
        let mut reader: [u8; 11] = [0; 11];
        let timestamp = random::<u32>() << 8 >> 8;
        let message_length = random::<u32>() << 8 >> 8;
        let message_type = random::<u8>();
        let message_id = random::<u32>();
        reader[..3].copy_from_slice(&timestamp.to_be_bytes()[1..]);
        reader[3..6].copy_from_slice(&message_length.to_be_bytes()[1..]);
        reader[6] = message_type;
        reader[7..].copy_from_slice(&message_id.to_le_bytes());
        let result = read_message_header(pin!(reader.as_slice()), New).await;
        assert!(result.is_ok());
        let message_header = result.unwrap();
        assert!(message_header.get_timestamp().is_some());
        assert!(message_header.get_message_length().is_some());
        assert!(message_header.get_message_type().is_some());
        assert!(message_header.get_message_id().is_some());
        assert_eq!(Duration::from_millis(timestamp as u64), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, message_header.get_message_length().unwrap());
        assert_eq!(MessageType::from(message_type), message_header.get_message_type().unwrap());
        assert_eq!(message_id, message_header.get_message_id().unwrap())
    }

    #[tokio::test]
    async fn read_same_source() {
        let mut reader: [u8; 7] = [0; 7];
        let timestamp = random::<u32>() << 8 >> 8;
        let message_length = random::<u32>() << 8 >> 8;
        let message_type = random::<u8>();
        reader[..3].copy_from_slice(&timestamp.to_be_bytes()[1..]);
        reader[3..6].copy_from_slice(&message_length.to_be_bytes()[1..]);
        reader[6] = message_type;
        let result = read_message_header(pin!(reader.as_slice()), SameSource).await;
        assert!(result.is_ok());
        let message_header = result.unwrap();
        assert!(message_header.get_timestamp().is_some());
        assert!(message_header.get_message_length().is_some());
        assert!(message_header.get_message_type().is_some());
        assert_eq!(Duration::from_millis(timestamp as u64), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, message_header.get_message_length().unwrap());
        assert_eq!(MessageType::from(message_type), message_header.get_message_type().unwrap())
    }

    #[tokio::test]
    async fn read_timer_change() {
        let mut reader: [u8; 3] = [0; 3];
        let timestamp = random::<u32>() << 8 >> 8;
        reader.copy_from_slice(&timestamp.to_be_bytes()[1..]);
        let result = read_message_header(pin!(reader.as_slice()), TimerChange).await;
        assert!(result.is_ok());
        let message_header = result.unwrap();
        assert!(message_header.get_timestamp().is_some());
        assert_eq!(Duration::from_millis(timestamp as u64), message_header.get_timestamp().unwrap())
    }

    #[tokio::test]
    async fn read_continue() {
        let reader: [u8; 0] = [0; 0];
        let result = read_message_header(pin!(reader.as_slice()), Continue).await.unwrap();
        assert!(result.get_timestamp().is_none())
    }
}
