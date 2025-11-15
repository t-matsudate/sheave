use std::{
    io::Result as IOResult,
    pin::Pin,
    task::{
        Context,
        Poll
    }
};
use tokio::{
    io::{
        AsyncRead,
        ReadBuf
    }
};

#[doc(hidden)]
#[derive(Debug)]
pub struct AwaitUntilReceiving<'a, R: AsyncRead + Unpin> {
    reader: &'a mut R
}

#[doc(hidden)]
impl<R: AsyncRead + Unpin> AsyncRead for AwaitUntilReceiving<'_, R> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<IOResult<()>> {
        loop {
            match Pin::new(&mut self.reader).poll_read(cx, buf) {
                Poll::Pending => continue,
                result => return result
            }
        }
    }
}

#[doc(hidden)]
pub fn await_until_receiving<'a, R: AsyncRead + Unpin>(reader: &'a mut R) -> AwaitUntilReceiving<'a, R> {
    AwaitUntilReceiving { reader }
}
