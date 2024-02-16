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
        amf::v0::AmfString,
        publishing_failure
    },
    readers::read_chunk,
    writers::write_chunk,
};

#[doc(hidden)]
#[derive(Debug)]
pub struct PublishHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for PublishHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        rtmp_context.increase_transaction_id();

        let transaction_id = rtmp_context.get_transaction_id();
        let publishing_name = rtmp_context.get_play_path().unwrap();
        let publish_request = Publish::new(
            transaction_id,
            publishing_name.clone(),
            "live".into()
        );
        let message_id = rtmp_context.get_message_id().unwrap();
        ready!(
            pin!(
                write_chunk(
                    self.0.as_mut(),
                    rtmp_context,
                    Duration::default(),
                    message_id,
                    &publish_request
                )
            ).poll(cx)
        )?;

        let stream_begin: StreamBegin = ready!(pin!(read_chunk(self.0.as_mut(), rtmp_context)).poll(cx))?;
        let stream_begin_message_id = stream_begin.get_message_id();

        if message_id != stream_begin_message_id {
            // TODO: Replace something logger later.
            println!("A property \"message ID\" is inconsistent. createStream result: {message_id}, Stream Begin: {stream_begin_message_id}.");
        }

        let on_status: OnStatus = ready!(pin!(read_chunk(self.0.as_mut(), rtmp_context)).poll(cx))?;

        // TODO: Replace something logger later.
        println!("information: {:?}", on_status.get_info_object());
        if on_status.get_info_object()["level"] == AmfString::from("error").into() {
            return Poll::Ready(Err(publishing_failure(on_status.into())))
        }

        rtmp_context.set_information(on_status.into());

        Poll::Ready(Ok(()))
    }
}

/// Handles a publish command as a client.
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
///             use sheave_client::handlers::handle_publish;
///
///             let mut stream = pin!(VecStream::default());
///             let mut client_rtmp_context = RtmpContext::default();
///             let play_path = AmfString::default();
///             let message_id = u32::default();
///             client_rtmp_context.set_transaction_id(4.into());
///             client_rtmp_context.set_play_path(play_path.clone());
///             client_rtmp_context.set_message_id(message_id);
///             let mut server_rtmp_context = RtmpContext::default();
///
///             // Because client handlers read response messages after request sent.
///             let stream_begin = StreamBegin::new(message_id);
///             ready!(pin!(write_chunk(stream.as_mut(), &mut RtmpContext::default(), Duration::default(), u32::default(), &stream_begin)).poll(cx))?;
///             // Same as above.
///             let on_status = OnStatus::new(
///                 object!(
///                     "level" => AmfString::from("status"),
///                     "code" => AmfString::from("NetStream.Publish.Start"),
///                     "description" => AmfString::new(format!("{play_path} is now published")),
///                     "details" => play_path.clone()
///                 )
///             );
///             ready!(pin!(write_chunk(stream.as_mut(), &mut RtmpContext::default(), Duration::default(), message_id, &on_status)).poll(cx))?;
///
///             ready!(pin!(handle_publish(stream.as_mut())).poll_handle(cx, &mut client_rtmp_context))?;
///
///             let actual: Publish = ready!(pin!(read_chunk(stream.as_mut(), &mut server_rtmp_context)).poll(cx))?;
///             let expected = Publish::new(5.into(), play_path, "live".into());
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
pub fn handle_publish<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> PublishHandler<'a, RW> {
    PublishHandler(stream)
}
