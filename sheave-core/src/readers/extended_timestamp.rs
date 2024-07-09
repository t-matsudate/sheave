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

#[doc(hidden)]
#[derive(Debug)]
pub struct ExtendedTimestampReader<'a, R: AsyncRead> {
    reader: Pin<&'a mut R>
}

#[doc(hidden)]
impl<R: AsyncRead> Future for ExtendedTimestampReader<'_, R> {
    type Output = IOResult<Duration>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        let mut extended_timestamp_bytes: [u8; 4] = [0; 4];
        let mut buf = ReadBuf::new(&mut extended_timestamp_bytes);
        ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
        Poll::Ready(Ok(Duration::from_millis(u32::from_be_bytes(extended_timestamp_bytes) as u64)))
    }
}

/// Reads an extended timestamp from streams.
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
/// use sheave_core::readers::read_extended_timestamp;
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let extended_timestamp = Duration::from_millis(random::<u32>() as u64);
///     let reader: [u8; 4] = (extended_timestamp.as_millis() as u32).to_be_bytes();
///     let result = read_extended_timestamp(pin!(reader.as_slice())).await?;
///     assert_eq!(extended_timestamp, result);
///     Ok(())
/// }
/// ```
pub fn read_extended_timestamp<R: AsyncRead>(reader: Pin<&mut R>) -> ExtendedTimestampReader<'_, R> {
    ExtendedTimestampReader { reader }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use rand::random;
    use super::*;

    #[tokio::test]
    async fn read_extended_ts() {
        let mut reader: [u8; 4] = [0; 4];
        let extended_timestamp = random::<u32>();
        reader.copy_from_slice(&extended_timestamp.to_be_bytes());
        let result = read_extended_timestamp(pin!(reader.as_slice())).await;
        assert!(result.is_ok());
        assert_eq!(Duration::from_millis(extended_timestamp as u64), result.unwrap())
    }
}
