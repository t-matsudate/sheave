use std::{
    io::Result as IOResult,
    pin::{
        Pin,
        pin
    },
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

/// The simple stream for std's buffer-like types.
///
/// `[u8]` can't write, against `Vec` can't read.
/// std's buffer-like types can't act as streams which coincide reading and writing.
/// Therefore this type was prepared for wrapping them.
/// Mainly, this is expected that is used for testing handlers.
#[derive(Debug, Default)]
pub struct VecStream(Vec<u8>);

impl AsyncRead for VecStream {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<IOResult<()>> {
        ready!(pin!(self.0.as_slice()).poll_read(cx, buf))?;
        self.0 = self.0.split_off(buf.filled().len());
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for VecStream {
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
