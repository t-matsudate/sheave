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
        ReleaseStream,
        ReleaseStreamResult,
        Command
    },
    readers::read_chunk,
    writers::write_chunk,
};

#[doc(hidden)]
#[derive(Debug)]
pub struct ReleaseStreamHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for ReleaseStreamHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        let release_stream_request: ReleaseStream = ready!(pin!(read_chunk(self.0.as_mut(), rtmp_context)).poll(cx))?;
        rtmp_context.set_transaction_id(release_stream_request.get_transaction_id());
        rtmp_context.set_play_path(release_stream_request.into());

        let release_stream_result = ReleaseStreamResult::new("_result".into(), rtmp_context.get_transaction_id());
        ready!(pin!(write_chunk(self.0.as_mut(), rtmp_context, Duration::default(), u32::default(), &release_stream_result)).poll(cx))?;
        Poll::Ready(Ok(()))
    }
}

/// Handles a releaseStream command as a server.
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
///                     ReleaseStream,
///                     ReleaseStreamResult,
///                     amf::v0::AmfString
///                 },
///                 readers::read_chunk,
///                 writers::write_chunk
///             };
///             use sheave_server::handlers::handle_release_stream;
///
///             let mut stream = pin!(VecStream::default());
///             let mut client_rtmp_context = RtmpContext::default();
///             let mut server_rtmp_context = RtmpContext::default();
///
///             ready!(pin!(write_chunk(stream.as_mut(), &mut client_rtmp_context, Duration::default(), u32::default(), &ReleaseStream::new(2.into(), AmfString::default()))).poll(cx))?;
///
///             ready!(pin!(handle_release_stream(stream.as_mut())).poll_handle(cx, &mut server_rtmp_context))?;
///
///             let actual: ReleaseStreamResult = ready!(pin!(read_chunk(stream.as_mut(), &mut client_rtmp_context)).poll(cx))?;
///             let expected = ReleaseStreamResult::new("_result".into(), 2.into());
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
pub fn handle_release_stream<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> ReleaseStreamHandler<'a, RW> {
    ReleaseStreamHandler(stream)
}
