use std::{
    io::Result as IOResult,
    pin::Pin,
    sync::Arc,
    task::{
        Context,
        Poll
    }
};
use tokio::io::{
    AsyncRead,
    AsyncWrite,
    ReadBuf
};

/// The wrapper for stream types.
#[derive(Debug)]
pub struct StreamWrapper<RW: Unpin>(RW);

impl<RW: Unpin> StreamWrapper<RW> {
    /// Constructs a wrapped stream.
    pub fn new(stream: RW) -> Self {
        Self(stream)
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
    pub fn make_weak_pin<'a>(self: &'a Arc<Self>) -> Pin<&'a mut RW> {
        unsafe { Pin::new(&mut *(Arc::downgrade(self).as_ptr() as *mut RW)) }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for StreamWrapper<R> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<IOResult<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for StreamWrapper<W> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IOResult<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IOResult<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IOResult<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
