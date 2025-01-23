use std::{
    io::Result as IOResult,
    future::Future,
    pin::{
        Pin,
        pin
    },
    task::{
        Context as FutureContext,
        Poll
    },
    time::Duration
};
use log::info;
use futures::ready;
use tokio::io::AsyncWrite;
use sheave_core::{
    ByteBuffer,
    Encoder,
    handlers::{
        AsyncHandler,
        MeasureAcknowledgement,
        RtmpContext,
        Middleware
    },
    messages::{
        Acknowledgement,
        ChunkData
    },
    writers::write_chunk
};

#[doc(hidden)]
#[derive(Debug)]
pub struct AcknowledgementWriter<'a, W: AsyncWrite + MeasureAcknowledgement + Unpin>(Pin<&'a mut W>);

#[doc(hidden)]
impl<W: AsyncWrite + MeasureAcknowledgement + Unpin> Middleware for AcknowledgementWriter<'_, W> {
    fn poll_handle_wrapped<H: AsyncHandler + Unpin>(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext, handler: Pin<&mut H>) -> Poll<IOResult<()>> {
        self.0.begin_measuring();

        ready!(handler.poll_handle(cx, rtmp_context))?;

        let acknowledgement = self.0.as_acknowledgement();

        self.0.finish_measuring();

        if acknowledgement > (rtmp_context.get_peer_bandwidth() / 8) {
            let mut buffer = ByteBuffer::default();
            buffer.encode(&acknowledgement);
            ready!(pin!(write_chunk(self.0.as_mut(), rtmp_context, Acknowledgement::CHANNEL.into(), Duration::default(), Acknowledgement::MESSAGE_TYPE, u32::default(), &Vec::<u8>::from(buffer))).poll(cx))?;
            info!("Acknowledgement got sent.");
        }

        Poll::Ready(Ok(()))
    }
}

#[doc(hidden)]
pub fn write_acknowledgement<'a, W: AsyncWrite + MeasureAcknowledgement + Unpin>(writer: Pin<&'a mut W>) -> AcknowledgementWriter<'a, W> {
    AcknowledgementWriter(writer)
}
