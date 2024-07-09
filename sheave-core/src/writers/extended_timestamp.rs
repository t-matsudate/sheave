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
use tokio::io::AsyncWrite;

#[doc(hidden)]
#[derive(Debug)]
pub struct ExtendedTimestampWriter<'a, W: AsyncWrite> {
    writer: Pin<&'a mut W>,
    extended_timestamp: Duration
}

#[doc(hidden)]
impl<W: AsyncWrite> Future for ExtendedTimestampWriter<'_, W> {
    type Output = IOResult<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        let extended_timestamp_bytes = (self.extended_timestamp.as_millis() as u32).to_be_bytes();
        self.writer.as_mut().poll_write(cx, &extended_timestamp_bytes).map_ok(|_| ())
    }
}

/// Writes an extended timestramp into streams.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::{
///         Pin,
///         pin
///     },
///     time::Duration
/// };
/// use rand::random;
/// use sheave_core::writers::write_extended_timestamp;
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let expected = Duration::from_millis(random::<u32>() as u64);
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     write_extended_timestamp(writer.as_mut(), expected).await?;
///     let mut written: [u8; 4] = [0; 4];
///     written.copy_from_slice(&writer[..4]);
///     let actual = Duration::from_millis(u32::from_be_bytes(written) as u64);
///     assert_eq!(expected, actual);
///     Ok(())
/// }
/// ```
pub fn write_extended_timestamp<W: AsyncWrite>(writer: Pin<&mut W>, extended_timestamp: Duration) -> ExtendedTimestampWriter<'_, W> {
    ExtendedTimestampWriter { writer, extended_timestamp }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use rand::random;
    use super::*;

    #[tokio::test]
    async fn write_extended_ts() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let expected = Duration::from_millis(random::<u32>() as u64);
        let result = write_extended_timestamp(writer.as_mut(), expected).await;
        assert!(result.is_ok());
        let mut written: [u8; 4] = [0; 4];
        written.copy_from_slice(&writer[..4]);
        let actual = Duration::from_millis(u32::from_be_bytes(written) as u64);
        assert_eq!(expected, actual)
    }
}
