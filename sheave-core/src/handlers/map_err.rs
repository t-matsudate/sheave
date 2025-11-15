use std::{
    io::{
        Error as IOError,
        Result as IOResult
    },
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

pub trait ErrorHandler {
    fn poll_handle_error(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext, error: IOError) -> Poll<IOResult<()>>;
}

pin_project! {
    #[doc(hidden)]
    #[derive(Debug)]
    pub struct MapErr<H, E> {
        #[pin] body: H,
        #[pin] error_handler: E
    }
}

#[doc(hidden)]
impl<H, E> AsyncHandler for MapErr<H, E>
where
    H: AsyncHandler + Unpin,
    E: ErrorHandler + Unpin
{
    fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        let this = self.project();
        if let Err(e) = ready!(this.body.poll_handle(cx, rtmp_context)) {
            this.error_handler.poll_handle_error(cx, rtmp_context, e)
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

#[doc(hidden)]
pub fn map_err<H, E>(body: H, error_handler: E) -> MapErr<H, E>
where
    H: AsyncHandler + Unpin,
    E: ErrorHandler + Unpin
{
    MapErr { body, error_handler }
}
