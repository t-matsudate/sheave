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
    pub struct WhileOk<H1, H2> {
        #[pin] before: H1,
        #[pin] condition: H2
    }
}

#[doc(hidden)]
impl<H, I> AsyncHandler for WhileOk<H, I>
where
    H: AsyncHandler + Unpin,
    I: AsyncHandler + Unpin
{
    fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        let mut this = self.project();

        ready!(this.before.poll_handle(cx, rtmp_context))?;

        loop {
            ready!(this.condition.as_mut().poll_handle(cx, rtmp_context))?;
        }
    }
}

pub fn while_ok<H1, H2>(before: H1, condition: H2) -> WhileOk<H1, H2>
where
    H1: AsyncHandler + Unpin,
    H2: AsyncHandler + Unpin
{
    WhileOk { before, condition }
}
