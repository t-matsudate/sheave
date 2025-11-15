use std::{
    io::Result as IOResult,
    pin::Pin,
    task::{
        Context as FutureContext,
        Poll
    }
};
use pin_project_lite::pin_project;
use futures::ready;
use super::{
    AsyncHandler,
    RtmpContext
};

pin_project! {
    #[doc(hidden)]
    #[derive(Debug)]
    pub struct Chain<H1, H2> {
        #[pin] current: H1,
        #[pin] next: H2
    }
}

#[doc(hidden)]
impl<H1, H2> AsyncHandler for Chain<H1, H2>
where
    H1: AsyncHandler + Unpin,
    H2: AsyncHandler + Unpin
{
    fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        let this = self.project();
        match ready!(this.current.poll_handle(cx, rtmp_context)) {
            Ok(_) => this.next.poll_handle(cx, rtmp_context),
            Err(e) => Poll::Ready(Err(e))
        }
    }
}

#[doc(hidden)]
pub fn chain<H1, H2>(current: H1, next: H2) -> Chain<H1, H2>
where
    H1: AsyncHandler + Unpin,
    H2: AsyncHandler + Unpin
{
    Chain { current, next }
}
