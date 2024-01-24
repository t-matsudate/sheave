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
        Connect,
        ConnectResult,
        amf::v0::{
            AmfString,
            Object
        }
    },
    object,
    readers::read_chunk,
    writers::write_chunk,
};

#[doc(hidden)]
#[derive(Debug)]
pub struct ConnectHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

#[doc(hidden)]
impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for ConnectHandler<'_, RW> {
    fn poll_handle(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
        rtmp_context.increase_transaction_id();

        let connect_request = Connect::new(
            object!(
                "app" => AmfString::from(""),
                "type" => AmfString::from("nonprivate"),
                "flashVer" => AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"),
                "tcUrl" => AmfString::from("")
            )
        );
        ready!(
            pin!(
                write_chunk(
                    self.0.as_mut(),
                    rtmp_context,
                    Duration::default(),
                    u32::default(),
                    &connect_request
                )
            ).poll(cx)
        )?;
        rtmp_context.set_command_object(connect_request.into());

        let connect_result: ConnectResult = ready!(pin!(read_chunk(self.0.as_mut(), rtmp_context)).poll(cx))?;
        let (properties, information): (Object, Object) = connect_result.into();
        rtmp_context.set_properties(properties);
        rtmp_context.set_information(information);
        Poll::Ready(Ok(()))
    }
}

/// Handles a connect command as a client.
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
///                     Connect,
///                     ConnectResult,
///                     amf::v0::{
///                         Number,
///                         AmfString
///                     }
///                 },
///                 object,
///                 readers::read_chunk,
///                 writers::write_chunk
///             };
///             use sheave_client::handlers::handle_connect;
///
///             let mut stream = pin!(VecStream::default());
///             let mut client_rtmp_context = RtmpContext::default();
///             let mut server_rtmp_context = RtmpContext::default();
///
///             // Because client handlers read response messages after request sent.
///             ready!(pin!(write_chunk(stream.as_mut(), &mut RtmpContext::default(), Duration::default(), u32::default(), &ConnectResult::default())).poll(cx))?;
///
///             ready!(pin!(handle_connect(stream.as_mut())).poll_handle(cx, &mut client_rtmp_context))?;
///
///             let actual: Connect = ready!(pin!(read_chunk(stream.as_mut(), &mut server_rtmp_context)).poll(cx))?;
///             let expected = Connect::new(
///                 object!(
///                     "app" => AmfString::from(""),
///                     "type" => AmfString::from("nonprivate"),
///                     "flashVer" => AmfString::from("FMLE/3.0 (compatible; Lavf 60.10.100)"),
///                     "tcUrl" => AmfString::from("")
///                 )
///             );
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
pub fn handle_connect<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> ConnectHandler<'a, RW> {
    ConnectHandler(stream)
}
