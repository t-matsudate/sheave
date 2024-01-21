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

#[derive(Debug)]
pub struct ConnectHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);

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

pub fn handle_connect<'a, RW: AsyncRead + AsyncWrite + Unpin>(stream: Pin<&'a mut RW>) -> ConnectHandler<'a, RW> {
    ConnectHandler(stream)
}
