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
        CreateStream,
        CreateStreamResult,
        Command
    },
    readers::read_chunk,
    writers::write_chunk,
};
use crate::server::provide_message_id;

#[doc(hidden)]
#[derive(Debug)]
pub struct CreateStreamHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for CreateStreamHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        let create_stream: CreateStream = ready!(pin!(read_chunk(self.0.as_mut(), rtmp_context)).poll(cx))?;
        let message_id = ready!(pin!(provide_message_id()).poll(cx));
        rtmp_context.set_transaction_id(create_stream.get_transaction_id());
        let create_stream_result: CreateStreamResult = CreateStreamResult::new("_result".into(), rtmp_context.get_transaction_id(), message_id.into());
        ready!(pin!(write_chunk(self.0.as_mut(), rtmp_context, Duration::default(), u32::default(), &create_stream_result)).poll(cx))?;
        rtmp_context.set_message_id(message_id.into());

        Poll::Ready(Ok(()))
    }
}

/// Handles a createStream command as a server.
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
///                     CreateStream,
///                     CreateStreamResult,
///                     amf::v0::Number
///                 },
///                 readers::read_chunk,
///                 writers::write_chunk
///             };
///             use sheave_server::handlers::handle_create_stream;
///
///             let mut stream = pin!(VecStream::default());
///             let mut client_rtmp_context = RtmpContext::default();
///             let mut server_rtmp_context = RtmpContext::default();
///
///             ready!(pin!(write_chunk(stream.as_mut(), &mut client_rtmp_context, Duration::default(), u32::default(), &CreateStream::new(4.into()))).poll(cx))?;
///
///             ready!(pin!(handle_create_stream(stream.as_mut())).poll_handle(cx, &mut server_rtmp_context))?;
///
///             let actual: CreateStreamResult = ready!(pin!(read_chunk(stream.as_mut(), &mut client_rtmp_context)).poll(cx))?;
///             let expected = CreateStreamResult::new("_result".into(), 4.into(), Number::default());
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
pub fn handle_create_stream<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> CreateStreamHandler<'a, RW> {
    CreateStreamHandler(stream)
}
