use std::{
    io::Result as IOResult,
    pin::Pin,
    task::{
        Context as FutureContext,
        Poll
    }
};
use pin_project_lite::pin_project;
use super::{
    AsyncHandler,
    RtmpContext
};

pub trait Middleware {
    fn poll_handle_wrapped<H: AsyncHandler + Unpin>(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext, handler: Pin<&mut H>) -> Poll<IOResult<()>>;
}

pin_project! {
    #[doc(hidden)]
    #[derive(Debug)]
    pub struct Wrap<M, H> {
        #[pin] middleware: M,
        #[pin] handler: H
    }
}

#[doc(hidden)]
impl<M, H> AsyncHandler for Wrap<M, H>
where
    M: Middleware + Unpin,
    H: AsyncHandler + Unpin
{
    fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        let this = self.project();
        this.middleware.poll_handle_wrapped(cx, rtmp_context, this.handler)
    }
}

#[doc(hidden)]
pub fn wrap<M, H>(middleware: M, handler: H) -> Wrap<M, H>
where
    M: Middleware + Unpin,
    H: AsyncHandler + Unpin
{
    Wrap { middleware, handler }
}
