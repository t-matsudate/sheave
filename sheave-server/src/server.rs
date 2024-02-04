mod message_id_provider;

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
    handle_second_handshake,
    handle_connect,
    handle_release_stream,
    handle_fc_publish,
    handle_create_stream
};
pub use self::message_id_provider::*;

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
                .chain(handle_connect(self.stream.make_weak_pin()))
                .chain(handle_release_stream(self.stream.make_weak_pin()))
                .chain(handle_fc_publish(self.stream.make_weak_pin()))
                .chain(handle_create_stream(self.stream.make_weak_pin()))
                .chain(echo_next(self.stream.make_weak_pin()))
        ).poll_handle(cx, self.rtmp_context.make_weak_mut())
    }
}

#[derive(Debug)]
struct EchoHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for EchoHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        use futures::ready;
        use sheave_core::readers::{
            read_basic_header,
            read_message_header,
            read_chunk_data
        };
        let basic_header = ready!(pin!(read_basic_header(self.0.as_mut())).poll(cx))?;
        println!("{basic_header:?}");
        let message_header = ready!(pin!(read_message_header(self.0.as_mut(), basic_header.get_message_format())).poll(cx))?;
        println!("{message_header:?}");
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let message_length = if let Some(message_length) = message_header.get_message_length() {
            message_length
        } else {
            rtmp_context.get_last_received_chunk(&basic_header.get_chunk_id()).unwrap().get_message_length()
        };
        let data = ready!(pin!(read_chunk_data(self.0.as_mut(), chunk_size, message_length)).poll(cx))?;
        println!("{data:?}");
        Poll::Ready(Ok(()))
    }
}

fn echo_next<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> EchoHandler<'a, RW> {
    EchoHandler(stream)
}
