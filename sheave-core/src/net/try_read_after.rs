use std::{
    future::Future,
    io::Result as IOResult,
    pin::{
        Pin,
        pin
    },
    task::{
        Context,
        Poll
    },
    time::Duration
};
use futures::ready;
use tokio::{
    io::{
        AsyncRead,
        ReadBuf
    },
    time::sleep
};

#[doc(hidden)]
#[derive(Debug)]
pub struct TryReadAfter<'a, R: AsyncRead + Unpin> {
    reader: &'a mut R,
    await_duration: Duration,
}

#[doc(hidden)]
impl<R: AsyncRead + Unpin> AsyncRead for TryReadAfter<'_, R> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<IOResult<()>> {
        ready!(pin!(sleep(self.await_duration)).poll(cx));

        Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}

#[doc(hidden)]
pub fn try_read_after<'a, R: AsyncRead + Unpin>(reader: &'a mut R, await_duration: Duration) -> TryReadAfter<'a, R> {
    TryReadAfter {
        reader,
        await_duration
    }
}
