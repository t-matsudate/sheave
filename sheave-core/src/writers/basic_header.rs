use std::{
    future::Future,
    io::Result as IOResult,
    pin::Pin,
    task::{
        Context as FutureContext,
        Poll
    }
};
use tokio::io::AsyncWrite;
use crate::messages::headers::BasicHeader;

#[doc(hidden)]
#[derive(Debug)]
pub struct BasicHeaderWriter<'a, W: AsyncWrite> {
    writer: Pin<&'a mut W>,
    basic_header: &'a BasicHeader
}

#[doc(hidden)]
impl<W: AsyncWrite> Future for BasicHeaderWriter<'_, W> {
    type Output = IOResult<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        let message_format = self.basic_header.get_message_format() as u8;
        let chunk_id = self.basic_header.get_chunk_id();
        let mut basic_header_bytes: Vec<u8> = Vec::new();
        if chunk_id >= 320 {
            basic_header_bytes.push(message_format << 6 | 1);
            basic_header_bytes.extend_from_slice((chunk_id - 64).to_le_bytes().as_slice());
        } else if chunk_id >= 64 {
            basic_header_bytes.push(message_format << 6);
            basic_header_bytes.push((chunk_id - 64) as u8);
        } else {
            basic_header_bytes.push(message_format << 6 | (chunk_id as u8));
        }
        self.writer.as_mut().poll_write(cx, basic_header_bytes.as_slice()).map_ok(|_| ())
    }
}

/// Writes a basic header into streams.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::{
///         Pin,
///         pin
///     }
/// };
/// use sheave_core::{
///     messages::headers::{
///         BasicHeader,
///         MessageFormat::*
///     },
///     writers::write_basic_header
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     // In case of 1 byte.
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let message_format = New;
///     let chunk_id = 2u16;
///     let basic_header = BasicHeader::new(message_format, chunk_id);
///     let result = write_basic_header(writer.as_mut(), &basic_header).await;
///     assert!(result.is_ok());
///     assert_eq!(message_format as u8, writer[0] >> 6);
///     assert_eq!(chunk_id, (writer[0] << 2 >> 2) as u16);
///
///     // In case of 2 bytes.
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let message_fomrat = New;
///     let chunk_id = 64u16;
///     let basic_header = BasicHeader::new(message_fomrat, chunk_id);
///     let result = write_basic_header(writer.as_mut(), &basic_header).await;
///     assert!(result.is_ok());
///     assert_eq!(message_format as u8, writer[0] >> 6);
///     assert_eq!(0, writer[0] << 2 >> 2);
///     assert_eq!(0, writer[1] as u16);
///
///     // In case of 3 bytes.
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let message_format = New;
///     let chunk_id = 320u16;
///     let basic_header = BasicHeader::new(message_format, chunk_id);
///     let result = write_basic_header(writer.as_mut(), &basic_header).await;
///     let mut written: [u8; 2] = [0; 2];
///     written.copy_from_slice(&writer[1..]);
///     let written = u16::from_le_bytes(written);
///     assert!(result.is_ok());
///     assert_eq!(message_format as u8, writer[0] >> 6);
///     assert_eq!(1, writer[0] << 2 >> 2);
///     assert_eq!(256, written);
///     Ok(())
/// }
/// ```
pub fn write_basic_header<'a, W: AsyncWrite>(writer: Pin<&'a mut W>, basic_header: &'a BasicHeader) -> BasicHeaderWriter<'a, W> {
    BasicHeaderWriter { writer, basic_header }
}

#[cfg(test)]
mod tests {
    use std::{
        cmp::max,
        pin::{
            Pin,
            pin
        }
    };
    use rand::random;
    use crate::messages::headers::{
        BasicHeader,
        MessageFormat
    };
    use super::*;

    #[tokio::test]
    async fn write_one_byte() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let byte = random::<u8>();
        let message_format: MessageFormat = (byte >> 6).into();
        let chunk_id = (byte << 2 >> 2) as u16;
        let basic_header = BasicHeader::new(message_format, chunk_id);
        let result = write_basic_header(writer.as_mut(), &basic_header).await;
        assert!(result.is_ok());
        assert_eq!(message_format as u8, writer[0] >> 6);
        assert_eq!(chunk_id, (writer[0] << 2 >> 2) as u16)
    }

    #[tokio::test]
    async fn write_two_bytes() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let message_format: MessageFormat = (random::<u8>() >> 6).into();
        let chunk_id = max(64, random::<u8>()) as u16;
        let basic_header = BasicHeader::new(message_format, chunk_id);
        let result = write_basic_header(writer.as_mut(), &basic_header).await;
        assert!(result.is_ok());
        assert_eq!(message_format as u8, writer[0] >> 6);
        assert_eq!(0, writer[0] << 2 >> 2);
        assert_eq!(chunk_id - 64, writer[1] as u16)
    }

    #[tokio::test]
    async fn write_three_bytes() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let message_format: MessageFormat = (random::<u8>() >> 6).into();
        let chunk_id = max(320, random::<u16>());
        let basic_header = BasicHeader::new(message_format, chunk_id);
        let result = write_basic_header(writer.as_mut(), &basic_header).await;
        assert!(result.is_ok());
        assert_eq!(message_format as u8, writer[0] >> 6);
        assert_eq!(1, writer[0] << 2 >> 2);
        let mut written: [u8; 2] = [0; 2];
        written.copy_from_slice(&writer[1..]);
        let written = u16::from_le_bytes(written);
        assert_eq!(chunk_id - 64, written)
    }
}
