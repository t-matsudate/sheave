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
        Command,
        amf::v0::{
            Number,
            AmfString
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
        let connect_request: Connect = ready!(pin!(read_chunk(self.0.as_mut(), rtmp_context)).poll(cx))?;
        rtmp_context.set_transaction_id(connect_request.get_transaction_id());
        rtmp_context.set_command_object(connect_request.into());
        let connect_result = ConnectResult::new(
            "_result".into(),
            object!(
                "fmsVer" => AmfString::from("FMS/5,0,17"),
                "capabilities" => Number::from(31)
            ),
            object!(
                "level" => AmfString::from("status"),
                "code" => AmfString::from("NetConnection.Connect.Success"),
                "description" => AmfString::from("Connection succeeded."),
                "objectEncoding" => Number::from(0)
            )
        );
        ready!(pin!(write_chunk(self.0.as_mut(), rtmp_context, Duration::default(), u32::default(), &connect_result)).poll(cx))?;
        Poll::Ready(Ok(()))
    }
}

/// Handles a connect command as a server.
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
///             use sheave_server::handlers::handle_connect;
///
///             let mut stream = pin!(VecStream::default());
///             let mut client_rtmp_context = RtmpContext::default();
///             let mut server_rtmp_context = RtmpContext::default();
///
///             ready!(pin!(write_chunk(stream.as_mut(), &mut client_rtmp_context, Duration::default(), u32::default(), &Connect::default())).poll(cx))?;
///
///             ready!(pin!(handle_connect(stream.as_mut())).poll_handle(cx, &mut server_rtmp_context))?;
///
///             let actual: ConnectResult = ready!(pin!(read_chunk(stream.as_mut(), &mut client_rtmp_context)).poll(cx))?;
///             let expected = ConnectResult::new(
///                 "_result".into(),
///                 object!(
///                     "fmsVer" => AmfString::from("FMS/5,0,17"),
///                     "capabilities" => Number::from(31)
///                 ),
///                 object!(
///                     "level" => AmfString::from("status"),
///                     "code" => AmfString::from("NetConnection.Connect.Success"),
///                     "description" => AmfString::from("Connection succeeded."),
///                     "objectEncoding" => Number::from(0)
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
