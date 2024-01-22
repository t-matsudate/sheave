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
        OnFcPublish
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
        let fc_publish: FcPublish = ready!(pin!(read_chunk(self.0.as_mut(), rtmp_context)).poll(cx))?;
        let release_stream_play_path = rtmp_context.get_play_path().unwrap();
        let fc_publish_play_path = fc_publish.get_play_path();

        if release_stream_play_path != fc_publish_play_path {
            // TODO: Replace something logger later.
            println!("A property \"play path\" is inconsistent. releaseStream: {release_stream_play_path}, FCPublish: {fc_publish_play_path}.");
        }

        ready!(pin!(write_chunk(self.0.as_mut(), rtmp_context, Duration::default(), u32::default(), &OnFcPublish)).poll(cx))?;
        Poll::Ready(Ok(()))
    }
}

/// Handles a FCPublish command as a server.
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
///                     amf::v0::AmfString
///                 },
///                 readers::read_chunk,
///                 writers::write_chunk
///             };
///             use sheave_server::handlers::handle_fc_publish;
///
///             let mut stream = pin!(VecStream::default());
///             let mut client_rtmp_context = RtmpContext::default();
///             let mut server_rtmp_context = RtmpContext::default();
///             server_rtmp_context.set_play_path(AmfString::default());
///
///             ready!(pin!(write_chunk(stream.as_mut(), &mut client_rtmp_context, Duration::default(), u32::default(), &FcPublish::new(3.into(), AmfString::default()))).poll(cx))?;
///
///             ready!(pin!(handle_fc_publish(stream.as_mut())).poll_handle(cx, &mut server_rtmp_context))?;
///
///             let actual: OnFcPublish = ready!(pin!(read_chunk(stream.as_mut(), &mut client_rtmp_context)).poll(cx))?;
///             assert_eq!(OnFcPublish, actual);
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
