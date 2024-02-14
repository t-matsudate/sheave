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
        OnStatus,
        Publish,
        StreamBegin,
        amf::v0::AmfString
    },
    object,
    readers::read_chunk,
    writers::write_chunk,
};

#[doc(hidden)]
#[derive(Debug)]
pub struct PublishHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for PublishHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        let publish: Publish = ready!(pin!(read_chunk(self.0.as_mut(), rtmp_context)).poll(cx))?;
        let (publishing_name, publishing_type): (AmfString, AmfString) = publish.into();
        let message_id = rtmp_context.get_message_id().unwrap().as_integer() as u32;
        let stream_begin = StreamBegin::new(message_id);
        let on_status = OnStatus::new(
            object!(
                "level" => AmfString::from("status"),
                "code" => AmfString::from("NetStream.Publish.Start"),
                "description" => AmfString::new(format!("{} is now published", publishing_name)),
                "details" => publishing_name.clone()
            )
        );
        ready!(pin!(write_chunk(self.0.as_mut(), rtmp_context, Duration::default(), u32::default(), &stream_begin)).poll(cx))?;
        ready!(pin!(write_chunk(self.0.as_mut(), rtmp_context, Duration::default(), message_id, &on_status)).poll(cx))?;
        rtmp_context.set_publishing_name(publishing_name);
        rtmp_context.set_publishing_type(publishing_type);

        Poll::Ready(Ok(()))
    }
}

/// Handles a publish command as a server.
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
///                     OnStatus,
///                     Publish,
///                     StreamBegin,
///                     amf::v0::AmfString
///                 },
///                 object,
///                 readers::read_chunk,
///                 writers::write_chunk
///             };
///             use sheave_server::handlers::handle_publish;
///
///             let mut stream = pin!(VecStream::default());
///             let mut client_rtmp_context = RtmpContext::default();
///             let mut server_rtmp_context = RtmpContext::default();
///             server_rtmp_context.set_message_id(0.into());
///             let publishing_name = AmfString::from("example");
///
///             ready!(pin!(write_chunk(stream.as_mut(), &mut client_rtmp_context, Duration::default(), u32::default(), &Publish::new(5.into(), publishing_name.clone(), "live".into()))).poll(cx))?;
///
///             ready!(pin!(handle_publish(stream.as_mut())).poll_handle(cx, &mut server_rtmp_context))?;
///
///             let actual_stream_begin: StreamBegin = ready!(pin!(read_chunk(stream.as_mut(), &mut client_rtmp_context)).poll(cx))?;
///             let actual_on_status: OnStatus = ready!(pin!(read_chunk(stream.as_mut(), &mut client_rtmp_context)).poll(cx))?;
///             let expected_stream_begin = StreamBegin::new(u32::default());
///             let expected_on_status = OnStatus::new(
///                 object!(
///                     "level" => AmfString::from("status"),
///                     "code" => AmfString::from("NetStream.Publish.Start"),
///                     "description" => AmfString::new(format!("{publishing_name} is now published")),
///                     "details" => publishing_name
///                 )
///             );
///             assert_eq!(expected_stream_begin, actual_stream_begin);
///             assert_eq!(expected_on_status, actual_on_status);
///
///             Poll::Ready(Ok(()))
///         }
///     ).await;
///     assert!(result.is_ok());
///
///     Ok(())
/// }
/// ```
pub fn handle_publish<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> PublishHandler<'a, RW> {
    PublishHandler(stream)
}
