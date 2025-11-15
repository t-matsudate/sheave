use std::{
    cmp::min,
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
use tokio::io::{
    AsyncRead,
    ReadBuf
};
use crate::messages::ChunkSize;
use super::read_basic_header;

#[doc(hidden)]
#[derive(Debug)]
pub struct ChunkDataReader<'a, R: AsyncRead> {
    reader: Pin<&'a mut R>,
    chunk_size: ChunkSize,
    message_length: u32
}

#[doc(hidden)]
impl<R: AsyncRead> Future for ChunkDataReader<'_, R> {
    type Output = IOResult<Vec<u8>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        unsafe {
            let mut chunk_data_bytes: Vec<u8> = Vec::new();
            let mut remained = self.message_length;

            loop {
                let capacity = min(self.chunk_size.get_chunk_size(), remained);
                let mut tmp_bytes: Vec<u8> = Vec::with_capacity(capacity as usize);
                tmp_bytes.set_len(tmp_bytes.capacity());
                let mut buf = ReadBuf::new(tmp_bytes.as_mut_slice());
                ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
                chunk_data_bytes.extend_from_slice(&tmp_bytes);

                remained -= capacity;
                if remained > 0 {
                    ready!(pin!(read_basic_header(self.reader.as_mut())).poll(cx))?;
                } else {
                    return Poll::Ready(Ok(chunk_data_bytes))
                }
            }
        }
    }
}

/// Reads a chunk data from streams.
///
/// If a chunk data exceeds specified chunk size, to insert continue headers between chunk data per chunk size is required.
/// Note the message length doesn't count their headers.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::pin
/// };
/// use rand::fill;
/// use sheave_core::{
///     messages::{
///         ChunkSize,
///         headers::MessageFormat
///     },
///     readers::read_chunk_data
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let chunk_size = ChunkSize::default();
///
///     // When it's just one chunk.
///     let mut reader: [u8; 128] = [0; 128];
///     fill(&mut reader);
///     let result = read_chunk_data(pin!(reader.as_slice()), chunk_size, 128).await?;
///     assert_eq!(128, result.len());
///
///     // When it has the one byte header.
///     let mut reader: [u8; 257] = [0; 257];
///     let mut part: [u8; 128] = [0; 128];
///     fill(&mut part);
///     reader[..128].copy_from_slice(&part);
///     reader[128] = u8::from(MessageFormat::Continue) << 6 | 2;
///     reader[129..].copy_from_slice(&part);
///     let result = read_chunk_data(pin!(reader.as_slice()), chunk_size, 256).await?;
///     assert_eq!(256, result.len());
///
///     // When it has the two bytes header.
///     let mut reader: [u8; 258] = [0; 258];
///     let mut part: [u8; 128] = [0; 128];
///     fill(&mut part);
///     reader[..128].copy_from_slice(&part);
///     reader[128] = u8::from(MessageFormat::Continue) << 6;
///     reader[129] = 2;
///     reader[130..].copy_from_slice(&part);
///     let result = read_chunk_data(pin!(reader.as_slice()), chunk_size, 256).await?;
///     assert_eq!(256, result.len());
///
///     // When it has the three bytes header.
///     let mut reader: [u8; 259] = [0; 259];
///     let mut part: [u8; 128] = [0; 128];
///     fill(&mut part);
///     reader[..128].copy_from_slice(&part);
///     reader[128] = u8::from(MessageFormat::Continue) << 6 | 1;
///     reader[129..131].copy_from_slice(&2u16.to_le_bytes());
///     reader[131..].copy_from_slice(&part);
///     let result = read_chunk_data(pin!(reader.as_slice()), chunk_size, 256).await?;
///     assert_eq!(256, result.len());
///
///     Ok(())
/// }
/// ```
pub fn read_chunk_data<'a, R: AsyncRead>(reader: Pin<&'a mut R>, chunk_size: ChunkSize, message_length: u32) -> ChunkDataReader<'a, R> {
    ChunkDataReader { reader, chunk_size, message_length }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use rand::fill;
    use crate::messages::headers::MessageFormat;
    use super::*;

    #[tokio::test]
    async fn read_one_chunk() {
        let mut reader: [u8; 128] = [0; 128];
        fill(&mut reader);
        let result = read_chunk_data(pin!(reader.as_slice()), ChunkSize::default(), 128).await;
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert_eq!(128, bytes.len())
    }

    #[tokio::test]
    async fn read_with_one_byte_header() {
        let mut reader: [u8; 257] = [0; 257];
        let mut part: [u8; 128] = [0; 128];
        fill(&mut part);
        reader[..128].copy_from_slice(&part);
        reader[128] = u8::from(MessageFormat::Continue) << 6 | 2;
        reader[129..].copy_from_slice(&part);
        let result = read_chunk_data(pin!(reader.as_slice()), ChunkSize::default(), 256).await;
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert_eq!(256, bytes.len())
    }

    #[tokio::test]
    async fn read_with_two_bytes_header() {
        let mut reader: [u8; 258] = [0; 258];
        let mut part: [u8; 128] = [0; 128];
        fill(&mut part);
        reader[..128].copy_from_slice(&part);
        reader[128] = u8::from(MessageFormat::Continue) << 6;
        reader[129] = 2;
        reader[130..].copy_from_slice(&part);
        let result = read_chunk_data(pin!(reader.as_slice()), ChunkSize::default(), 256).await;
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert_eq!(256, bytes.len())
    }

    #[tokio::test]
    async fn read_with_three_bytes_header() {
        let mut reader: [u8; 259] = [0; 259];
        let mut part: [u8; 128] = [0; 128];
        fill(&mut part);
        reader[..128].copy_from_slice(&part);
        reader[128] = u8::from(MessageFormat::Continue) << 6 | 1;
        reader[129..131].copy_from_slice(&2u16.to_le_bytes());
        reader[131..].copy_from_slice(&part);
        let result = read_chunk_data(pin!(reader.as_slice()), ChunkSize::default(), 256).await;
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert_eq!(256, bytes.len())
    }
}
