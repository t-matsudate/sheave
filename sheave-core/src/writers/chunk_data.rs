use std::{
    future::Future,
    io::Result as IOResult,
    pin::{
        Pin,
        pin
    },
    task::{
        Context as FutureContext,
        Poll
    }
};
use futures::ready;
use tokio::io::AsyncWrite;
use crate::{
    messages::{
        ChunkSize,
        headers::{
            BasicHeader,
            MessageFormat
        }
    },
    writers::write_basic_header
};

#[doc(hidden)]
#[derive(Debug)]
pub struct ChunkDataWriter<'a, W: AsyncWrite> {
    writer: Pin<&'a mut W>,
    chunk_id: u16,
    chunk_size: ChunkSize,
    chunk_data: &'a [u8],
}

#[doc(hidden)]
impl<W: AsyncWrite> Future for ChunkDataWriter<'_, W> {
    type Output = IOResult<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        let chunk_size = self.chunk_size.get_chunk_size();
        let mut chunks = self.chunk_data.chunks(chunk_size as usize);
        while let Some(chunk) = chunks.next() {
            ready!(self.writer.as_mut().poll_write(cx, chunk))?;

            if chunks.size_hint().0 >= 1 {
                let basic_header = BasicHeader::new(MessageFormat::Continue, self.chunk_id);
                ready!(pin!(write_basic_header(self.writer.as_mut(), &basic_header)).poll(cx))?;
            }
        }

        Poll::Ready(Ok(()))
    }
}

/// Writes a chunk data into streams.
///
/// If a chunk data exceeds specified chunk size, continue headers is inserted between chunk data per chunk size.
/// Note the message length doesn't count their headers.
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
/// use rand::fill;
/// use sheave_core::{
///     messages::{
///         ChunkSize,
///         headers::MessageFormat
///     },
///     writers::write_chunk_data
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     // When it's just one chunk.
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let mut chunk_data: [u8; 128] = [0; 128];
///     fill(&mut chunk_data);
///     write_chunk_data(writer.as_mut(), 2, ChunkSize::default(), &chunk_data).await?;
///     assert_eq!(128, writer.len());
///
///     // When it requires the one byte header.
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let mut chunk_data: [u8; 256] = [0; 256];
///     fill(&mut chunk_data);
///     write_chunk_data(writer.as_mut(), 2, ChunkSize::default(), &chunk_data).await?;
///     assert_eq!(257, writer.len());
///     let message_format: MessageFormat = (writer[128] >> 6).into();
///     assert_eq!(MessageFormat::Continue, message_format);
///     let chunk_id = writer[128] << 2 >> 2;
///     assert_eq!(2, chunk_id);
///
///     // When it requires the two bytes header.
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let mut chunk_data: [u8; 256] = [0; 256];
///     fill(&mut chunk_data);
///     write_chunk_data(writer.as_mut(), 64, ChunkSize::default(), &chunk_data).await?;
///     assert_eq!(258, writer.len());
///     let message_format: MessageFormat = (writer[128] >> 6).into();
///     assert_eq!(MessageFormat::Continue, message_format);
///     assert_eq!(0, writer[128] << 2 >> 2);
///     let chunk_id = writer[129];
///     assert_eq!(64, chunk_id + 64);
///
///     // When it requires the three bytes header.
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let mut chunk_data: [u8; 256] = [0; 256];
///     fill(&mut chunk_data);
///     write_chunk_data(writer.as_mut(), 320, ChunkSize::default(), &chunk_data).await?;
///     assert_eq!(259, writer.len());
///     let message_format: MessageFormat = (writer[128] >> 6).into();
///     assert_eq!(MessageFormat::Continue, message_format);
///     assert_eq!(1, writer[128] << 2 >> 2);
///     let mut chunk_id_bytes: [u8; 2] = [0; 2];
///     chunk_id_bytes.copy_from_slice(&writer[129..131]);
///     let chunk_id = u16::from_le_bytes(chunk_id_bytes);
///     assert_eq!(320, chunk_id + 64);
///
///     Ok(())
/// }
/// ```
pub fn write_chunk_data<'a, W: AsyncWrite>(writer: Pin<&'a mut W>, chunk_id: u16, chunk_size: ChunkSize, chunk_data: &'a [u8]) -> ChunkDataWriter<'a, W> {
    ChunkDataWriter { writer, chunk_id, chunk_size, chunk_data }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use rand::fill;
    use super::*;

    #[tokio::test]
    async fn write_one_chunk() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let mut chunk_data: [u8; 128] = [0; 128];
        fill(&mut chunk_data);
        let result = write_chunk_data(writer.as_mut(), 2, ChunkSize::default(), &chunk_data).await;
        assert!(result.is_ok());
        assert_eq!(128, writer.len())
    }

    #[tokio::test]
    async fn write_with_one_byte_header() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let mut chunk_data: [u8; 256] = [0; 256];
        fill(&mut chunk_data);
        let result = write_chunk_data(writer.as_mut(), 2, ChunkSize::default(), &chunk_data).await;
        assert!(result.is_ok());
        assert_eq!(257, writer.len())
    }

    #[tokio::test]
    async fn write_with_two_bytes_header() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let mut chunk_data: [u8; 256] = [0; 256];
        fill(&mut chunk_data);
        let result = write_chunk_data(writer.as_mut(), 64, ChunkSize::default(), &chunk_data).await;
        assert!(result.is_ok());
        assert_eq!(258, writer.len())
    }

    #[tokio::test]
    async fn write_with_three_bytes_header() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let mut chunk_data: [u8; 256] = [0; 256];
        fill(&mut chunk_data);
        let result = write_chunk_data(writer.as_mut(), 320, ChunkSize::default(), &chunk_data).await;
        assert!(result.is_ok());
        assert_eq!(259, writer.len());
    }
}
