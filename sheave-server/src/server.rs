use std::{
    future::Future,
    io::{
        Result as IOResult
    },
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
use sheave_core::{
    handlers::{
        AsyncHandler,
        AsyncHandlerExt,
        RtmpContext,
        StreamWrapper
    }
};
use crate::handlers::{
    handle_first_handshake,
    handle_second_handshake
};

#[derive(Debug)]
pub struct Server<RW: AsyncRead + AsyncWrite + Unpin> {
    stream: Arc<StreamWrapper<RW>>,
    rtmp_context: Arc<RtmpContext>
}

impl<RW: AsyncRead + AsyncWrite + Unpin> Server<RW> {
    pub fn new(stream: RW) -> Self {
        Self {
            stream: Arc::new(StreamWrapper::new(stream)),
            rtmp_context: Arc::new(RtmpContext::default())
        }
    }
}

impl<RW: AsyncRead + AsyncWrite + Unpin> Future for Server<RW> {
    type Output = IOResult<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        pin!(
            handle_first_handshake(self.stream.make_weak_pin())
                .chain(handle_second_handshake(self.stream.make_weak_pin()))
        ).poll_handle(cx, self.rtmp_context.make_weak_mut())
    }
}
