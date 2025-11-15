use std::{
    future::Future,
    io::Result as IOResult,
    pin::Pin,
    task::{
        Context as FutureContext,
        Poll
    }
};
use futures::ready;
use tokio::io::{
    AsyncRead,
    ReadBuf
};
use crate::messages::headers::{
    BasicHeader,
    MessageFormat
};

#[doc(hidden)]
#[derive(Debug)]
pub struct BasicHeaderReader<'a, R: AsyncRead> {
    reader: Pin<&'a mut R>
}

#[doc(hidden)]
impl<R: AsyncRead> Future for BasicHeaderReader<'_, R> {
    type Output = IOResult<BasicHeader>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        let mut first_byte: [u8; 1] = [0];
        let mut buf = ReadBuf::new(&mut first_byte);
        ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
        let message_format: MessageFormat = (first_byte[0] >> 6).into();
        let chunk_id = match first_byte[0] << 2 >> 2 {
            1 => {
                let mut chunk_id_bytes: [u8; 2] = [0; 2];
                let mut buf = ReadBuf::new(&mut chunk_id_bytes);
                ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
                u16::from_le_bytes(chunk_id_bytes) + 64
            },
            0 => {
                let mut chunk_id_bytes: [u8; 2] = [0; 2];
                let mut buf = ReadBuf::new(&mut chunk_id_bytes[1..]);
                ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
                u16::from_be_bytes(chunk_id_bytes) + 64
            },
            chunk_id => chunk_id as u16
        };
        Poll::Ready(Ok(BasicHeader::new(message_format, chunk_id)))
    }
}

/// Reads a basic header from streams.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::pin
/// };
/// use sheave_core::{
///     messages::headers::{
///         BasicHeader,
///         MessageFormat::*
///     },
///     readers::read_basic_header
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     // In case of 1 byte.
///     let reader = ((New as u8) << 6 | 2).to_be_bytes();
///     let result = read_basic_header(pin!(reader.as_slice())).await?;
///     assert_eq!(New, result.get_message_format());
///     assert_eq!(2, result.get_chunk_id());
///
///     // In case of 2 bytes.
///     let mut reader: [u8; 2] = [0; 2];
///     reader[0] = (New as u8) << 6;
///     reader[1] = 2;
///     let result = read_basic_header(pin!(reader.as_slice())).await?;
///     assert_eq!(New, result.get_message_format());
///     assert_eq!(66, result.get_chunk_id());
///
///     // In case of 3 bytes.
///     let mut reader: [u8; 3] = [0; 3];
///     reader[0] = (New as u8) << 6 | 1;
///     reader[1..].copy_from_slice((2 as u16).to_le_bytes().as_slice());
///     let result = read_basic_header(pin!(reader.as_slice())).await?;
///     assert_eq!(New, result.get_message_format());
///     assert_eq!(66, result.get_chunk_id());
///     Ok(())
/// }
/// ```
pub fn read_basic_header<R: AsyncRead>(reader: Pin<&mut R>) -> BasicHeaderReader<'_, R> {
    BasicHeaderReader { reader }
}

#[cfg(test)]
mod tests {
    use std::{
        cmp::max,
        pin::pin
    };
    use rand::random;
    use crate::messages::headers::MessageFormat;
    use super::*;

    #[tokio::test]
    async fn read_one_byte() {
        let message_format_bits = random::<u8>() & 0xc0;
        let chunk_id_bits = max(2, random::<u8>() << 2 >> 2);
        let reader: [u8; 1] = [message_format_bits | chunk_id_bits];
        let result = read_basic_header(pin!(reader.as_slice())).await;
        assert!(result.is_ok());
        let basic_header = result.unwrap();
        assert_eq!(MessageFormat::from(message_format_bits >> 6), basic_header.get_message_format());
        assert_eq!(chunk_id_bits as u16, basic_header.get_chunk_id())
    }

    #[tokio::test]
    async fn read_two_bytes() {
        let message_format_bits = random::<u8>() & 0xc0;
        let is_two_bytes: u8 = 0;
        let chunk_id_byte = random::<u8>();
        let mut reader: [u8; 2] = [0; 2];
        reader[0] = message_format_bits | is_two_bytes;
        reader[1] = chunk_id_byte;
        let result = read_basic_header(pin!(reader.as_slice())).await;
        assert!(result.is_ok());
        let basic_header = result.unwrap();
        assert_eq!(MessageFormat::from(message_format_bits >> 6), basic_header.get_message_format());
        assert_eq!((chunk_id_byte as u16) + 64, basic_header.get_chunk_id())
    }

    #[tokio::test]
    async fn read_three_bytes() {
        let message_format_bits = random::<u8>() & 0xc0;
        let is_two_bytes: u8 = 1;
        let chunk_id_bytes = random::<u16>();
        let mut reader: [u8; 3] = [0; 3];
        reader[0] = message_format_bits | is_two_bytes;
        reader[1..].copy_from_slice(chunk_id_bytes.to_le_bytes().as_slice());
        let result = read_basic_header(pin!(reader.as_slice())).await;
        assert!(result.is_ok());
        let basic_header = result.unwrap();
        assert_eq!(MessageFormat::from(message_format_bits >> 6), basic_header.get_message_format());
        assert_eq!(chunk_id_bytes + 64, basic_header.get_chunk_id())
    }
}
