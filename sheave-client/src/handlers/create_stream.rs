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
        CreateStreamResult
    },
    readers::read_chunk,
    writers::write_chunk,
};

#[doc(hidden)]
#[derive(Debug)]
pub struct CreateStreamHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for CreateStreamHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        rtmp_context.increase_transaction_id();

        let create_stream_request =  CreateStream::new(rtmp_context.get_transaction_id());
        ready!(
            pin!(
                write_chunk(
                    self.0.as_mut(),
                    rtmp_context,
                    Duration::default(),
                    u32::default(),
                    &create_stream_request
                )
            ).poll(cx)
        )?;

        let create_stream_result: CreateStreamResult = ready!(pin!(read_chunk(self.0.as_mut(), rtmp_context)).poll(cx))?;
        rtmp_context.set_message_id(create_stream_result.into());

        Poll::Ready(Ok(()))
    }
}

/// Handles a createStream command as a client.
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
///             use sheave_client::handlers::handle_create_stream;
///
///             let mut stream = pin!(VecStream::default());
///             let mut client_rtmp_context = RtmpContext::default();
///             client_rtmp_context.set_transaction_id(3.into());
///             let mut server_rtmp_context = RtmpContext::default();
///
///             // Because client handlers read response messages after request sent.
///             ready!(pin!(write_chunk(stream.as_mut(), &mut RtmpContext::default(), Duration::default(), u32::default(), &CreateStreamResult::new("_result".into(), 4.into(), Number::default()))).poll(cx))?;
///
///             ready!(pin!(handle_create_stream(stream.as_mut())).poll_handle(cx, &mut client_rtmp_context))?;
///
///             let actual: CreateStream = ready!(pin!(read_chunk(stream.as_mut(), &mut server_rtmp_context)).poll(cx))?;
///             let expected = CreateStream::new(4.into());
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
