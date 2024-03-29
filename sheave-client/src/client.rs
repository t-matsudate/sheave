use std::{
    future::Future,
    io::Result as IOResult,
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
    AsyncHandler,
    AsyncHandlerExt,
    RtmpContext,
    StreamWrapper
};
use crate::handlers::{
    handle_first_handshake,
    handle_second_handshake,
    handle_connect,
    handle_release_stream,
    handle_fc_publish,
    handle_create_stream,
    handle_publish
};

#[derive(Debug)]
pub struct Client<RW: AsyncRead + AsyncWrite + Unpin> {
    stream: Arc<StreamWrapper<RW>>,
    rtmp_context: Arc<RtmpContext>
}

impl<RW: AsyncRead + AsyncWrite + Unpin> Client<RW> {
    pub fn new(stream: RW) -> Self {
        Self {
            stream: Arc::new(StreamWrapper::new(stream)),
            rtmp_context: Arc::new(RtmpContext::default())
        }
    }
}

impl<RW: AsyncRead + AsyncWrite + Unpin> Future for Client<RW> {
    type Output = IOResult<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        pin!(
            handle_first_handshake(self.stream.make_weak_pin())
                .chain(handle_second_handshake(self.stream.make_weak_pin()))
                .chain(handle_connect(self.stream.make_weak_pin()))
                .chain(handle_release_stream(self.stream.make_weak_pin()))
                .chain(handle_fc_publish(self.stream.make_weak_pin()))
                .chain(handle_create_stream(self.stream.make_weak_pin()))
                .chain(handle_publish(self.stream.make_weak_pin()))
        ).poll_handle(cx, self.rtmp_context.make_weak_mut())
    }
}
