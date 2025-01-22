use std::{
    io::Result as IOResult,
    pin::Pin,
    sync::Arc,
    task::{
        Context,
        Poll
    }
};
use futures::ready;
use tokio::io::{
    AsyncRead,
    AsyncWrite,
    ReadBuf
};
use super::MeasureAcknowledgement;

/// The wrapper for stream types.
#[derive(Debug)]
pub struct StreamWrapper<RW: Unpin> {
    stream: RW,
    is_measured: bool,
    current_amount: u32
}

impl<RW: Unpin> StreamWrapper<RW> {
    /// Constructs a wrapped stream.
    pub fn new(stream: RW) -> Self {
        Self {
            stream,
            is_measured: bool::default(),
            current_amount: u32::default()
        }
    }

    /// Makes this stream into *pinned* weak pointer.
    ///
    /// Currently upper APIs use this wrapper via `Arc`.
    /// Because avoids problems which every RTMP's connection steps need same stream but can't borrow mutablly across scopes.
    /// Therefore upper APIs wrap streams into `Arc` at first, then make them able to copy as weak pointers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use sheave_core::handlers::{
    ///     StreamWrapper,
    ///     VecStream
    /// };
    ///
    /// Arc::new(StreamWrapper::new(VecStream::default())).make_weak_pin();
    /// ```
    pub fn make_weak_pin<'a>(self: &'a Arc<Self>) -> Pin<&'a mut Self> {
        unsafe { Pin::new(&mut *(Arc::downgrade(self).as_ptr() as *mut Self)) }
    }
}

impl<RW: Unpin> MeasureAcknowledgement for StreamWrapper<RW> {
    fn begin_measuring(&mut self) {
        self.current_amount = u32::default();
        self.is_measured = true;
    }

    fn finish_measuring(&mut self) {
        self.current_amount = u32::default();
        self.is_measured = false;
    }

    fn add_amount(&mut self, amount: u32) {
        self.current_amount += amount;
    }

    fn get_current_amount(&mut self) -> u32 {
        self.current_amount
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for StreamWrapper<R> {
    /// Wraps a stream to make it able to measure the amount of bytes.
    ///
    /// When bytes read exceeded some bandwidth limit, RTMP peers are required to send the `Acknowldgement` message to the other peer.
    /// But prepared stream like Vec, slice, or TCP streams has no implementation above.
    /// Therefore, StreamWrapper measures amounts of bytes read and writes `Acknowledgement` messages instead.
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<IOResult<()>> {
        ready!(Pin::new(&mut self.stream).poll_read(cx, buf))?;

        if self.is_measured {
            self.add_amount(buf.filled().len() as u32);
        }

        Poll::Ready(Ok(()))
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for StreamWrapper<W> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IOResult<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IOResult<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IOResult<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
