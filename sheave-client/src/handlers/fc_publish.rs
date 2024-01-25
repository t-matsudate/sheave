use std::{
    future::Future,
    io::Result as IOResult,
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
use futures::ready;
use tokio::io::{
    AsyncRead,
    AsyncWrite
};
use sheave_core::{
    handlers::{
        AsyncHandler,
        RtmpContext
    },
    messages::{
        FcPublish,
        OnFcPublish,
        amf::v0::AmfString
    },
    readers::read_chunk,
    writers::write_chunk,
};

#[doc(hidden)]
#[derive(Debug)]
pub struct FcPublishHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for FcPublishHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        rtmp_context.increase_transaction_id();

        let fc_publish_request = FcPublish::new(
            rtmp_context.get_transaction_id(),
            rtmp_context.get_play_path().map_or(
                AmfString::default(),
                |play_path| play_path.clone()
            )
        );
        ready!(
            pin!(
                write_chunk(
                    self.0.as_mut(),
                    rtmp_context,
                    Duration::default(),
                    u32::default(),
                    &fc_publish_request
                )
            ).poll(cx)
        )?;

        let _: OnFcPublish = ready!(pin!(read_chunk(self.0.as_mut(), rtmp_context)).poll(cx))?;
        Poll::Ready(Ok(()))
    }
}

/// Handles a FCPublish command as a client.
///
/// # Examples
///
/// ```rust
/// use std::io::Result as IOResult;
/// use futures::future::poll_fn;
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let result: IOResult<()> = poll_fn(
///         |cx| {
///             use std::{
///                 future::Future,
///                 pin::pin,
///                 task::Poll,
///                 time::Duration
///             };
///             use futures::ready;
///             use sheave_core::{
///                 handlers::{
///                     AsyncHandler,
///                     RtmpContext,
///                     VecStream
///                 },
///                 messages::{
///                     FcPublish,
///                     OnFcPublish,
///                 },
///                 readers::read_chunk,
///                 writers::write_chunk
///             };
///             use sheave_client::handlers::handle_fc_publish;
///
///             let mut stream = pin!(VecStream::default());
///             let mut client_rtmp_context = RtmpContext::default();
///             client_rtmp_context.set_transaction_id(2.into());
///             let mut server_rtmp_context = RtmpContext::default();
///
///             // Because client handlers read response messages after request sent.
///             ready!(pin!(write_chunk(stream.as_mut(), &mut RtmpContext::default(), Duration::default(), u32::default(), &OnFcPublish)).poll(cx))?;
///
///             ready!(pin!(handle_fc_publish(stream.as_mut())).poll_handle(cx, &mut client_rtmp_context))?;
///
///             let actual: FcPublish = ready!(pin!(read_chunk(stream.as_mut(), &mut server_rtmp_context)).poll(cx))?;
///             let expected = FcPublish::new(3.into(), "".into());
///             assert_eq!(expected, actual);
///
///             Poll::Ready(Ok(()))
///         }
///     ).await;
///     assert!(result.is_ok());
///
///     Ok(())
/// }
/// ```
pub fn handle_fc_publish<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> FcPublishHandler<'a, RW> {
    FcPublishHandler(stream)
}
