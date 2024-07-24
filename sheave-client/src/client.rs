use std::{
    future::Future,
    io::Result as IOResult,
    marker::PhantomData,
    pin::{
        Pin,
        pin
    },
    sync::Arc,
    task::{
        Context as FutureContext,
        Poll
    }
};
use tokio::io::{
    AsyncRead,
    AsyncWrite
};
use sheave_core::handlers::{
    HandlerConstructor,
    RtmpContext,
    StreamWrapper
};

#[derive(Debug)]
pub struct Client<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin,
    C: HandlerConstructor<StreamWrapper<RW>>
{
    stream: Arc<StreamWrapper<RW>>,
    rtmp_context: Arc<RtmpContext>,
    handler_constructor: PhantomData<C>
}

impl<RW, C> Client<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin,
    C: HandlerConstructor<StreamWrapper<RW>>
{
    pub fn new(stream: RW, rtmp_context: RtmpContext, handler_constructor: PhantomData<C>) -> Self {
        Self {
            stream: Arc::new(StreamWrapper::new(stream)),
            rtmp_context: Arc::new(rtmp_context),
            handler_constructor
        }
    }
}

impl<RW, C> Future for Client<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin,
    C: HandlerConstructor<StreamWrapper<RW>>
{
    type Output = IOResult<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        pin!(C::new(Arc::clone(&self.stream))).poll_handle(cx, self.rtmp_context.make_weak_mut())
    }
}
