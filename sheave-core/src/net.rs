pub mod rtmp;
mod await_until_receiving;
mod try_read_after;

use std::time::Duration;
use tokio::io::AsyncRead;
use self::{
    await_until_receiving::*,
    try_read_after::*
};

/// Reader extensions for RTMP.
///
/// In almost cases, the network communication is enough just to await until receiving some message.
/// But RTMP clients are required to be able to read messages both `Acknowledgement` and stream closing from servers, and these messages aren't necessarily always sent.
/// In this moment, clients will be stayed their processings if await receiving forever.
///
/// For solving above timing mismatches, to prepare several choices to receive chunks are required.
///
/// This trait provides several flexibility to read chunk by preparing following methods.
///
/// * [`await_until_receiving`]: The default of receiving behavior.
/// * [`try_read_after`]: Currently for clients.
///
/// [`await_until_receiving`]: RtmpReadExt::await_until_receiving
/// [`try_read_after`]: RtmpReadExt::try_read_after
pub trait RtmpReadExt: AsyncRead {
    /// Makes a stream awaiting until receiving some message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tokio::io::{
    ///     AsyncRead,
    ///     AsyncReadExt,
    ///     AsyncWrite,
    ///     AsyncWriteExt
    /// };
    /// use sheave_core::{
    ///     handlers::VecStream,
    ///     net::RtmpReadExt
    /// };
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut stream = VecStream::default();
    ///     stream.write_u8(1).await.unwrap();
    ///     assert!(stream.await_until_receiving().read_u8().await.is_ok())
    /// }
    /// ```
    fn await_until_receiving<'a>(&'a mut self) -> AwaitUntilReceiving<'a, Self>
    where Self: Sized + Unpin
    {
        await_until_receiving(self)
    }

    /// Makes a stream sleeping during specified duration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use tokio::io::{
    ///     AsyncRead,
    ///     AsyncReadExt,
    ///     AsyncWrite,
    ///     AsyncWriteExt
    /// };
    /// use sheave_core::{
    ///     handlers::VecStream,
    ///     net::RtmpReadExt
    /// };
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut stream = VecStream::default();
    ///     stream.write_u8(1).await.unwrap();
    ///     assert!(stream.try_read_after(Duration::from_secs(1)).read_u8().await.is_ok())
    /// }
    /// ```
    fn try_read_after<'a>(&'a mut self, await_duration: Duration) -> TryReadAfter<'a, Self>
    where Self: Sized + Unpin
    {
        try_read_after(self, await_duration)
    }
}

impl<R: AsyncRead> RtmpReadExt for R {}
