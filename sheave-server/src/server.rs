use std::{
    future::Future,
    io::{
        Result as IOResult
    },
    marker::PhantomData,
    pin::{
        Pin,
        pin
    },
    sync::Arc,
    task::{
        Context as FutureContext,
        Poll
    }
};
use tokio::io::{
    AsyncRead,
    AsyncWrite
};
use sheave_core::handlers::{
    RtmpContext,
    StreamWrapper,
    HandlerConstructor
};

/// The server instance of the Sheave
///
/// This consists of:
///
/// * Some stream instance which can both of read and write.
/// * Context data in the server.
/// * Some type parameter which implemented the [`HandlerConstructor`] trait.
///
/// The server wraps streams into [`Arc`] as a way of sharing streams among communication steps.
/// And also wraps contexts because of the same purpose.
///
/// The server makes any foreign handler to be able to construct via the [`PhantomData`], where a type parameter of [`PhantomData`] requires to implement the [`HandlerConstructor`] trait.
/// That is, its type parameter behaves as the constructor injection.
///
/// ## Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     marker::PhantomData,
///     pin::Pin,
///     sync::Arc,
///     task::{
///         Context as FutureContext,
///         Poll
///     }
/// };
/// use tokio::io::{
///     AsyncRead,
///     AsyncWrite
/// };
/// use sheave_core::handlers::{
///     AsyncHandler,
///     HandlerConstructor,
///     RtmpContext,
///     StreamWrapper,
///     VecStream
/// };
/// use sheave_server::Server;
///
/// struct SomethingHandler<RW: AsyncRead + AsyncWrite + Unpin>(Arc<StreamWrapper<RW>>);
///
/// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for SomethingHandler<RW> {
///     fn poll_handle(self: Pin<&mut Self>, _cx: &mut FutureContext<'_>, _rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
///         Poll::Ready(Ok(()))
///     }
/// }
///
/// impl<RW: AsyncRead + AsyncWrite + Unpin> HandlerConstructor<StreamWrapper<RW>> for SomethingHandler<RW> {
///     fn new(stream: Arc<StreamWrapper<RW>>) -> Self {
///         Self(stream)
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let stream = VecStream::default();
///     let rtmp_context = RtmpContext::default();
///     let mut server = Server::new(stream, rtmp_context, PhantomData::<SomethingHandler<VecStream>>);
///     let result = server.await;
///     assert!(result.is_ok())
/// }
/// ```
///
/// [`Arc`]: std::sync::Arc
/// [`PhantomData`]: std::marker::PhantomData
/// [`HandlerConstructor`]: sheave_core::handlers::HandlerConstructor
#[derive(Debug)]
pub struct Server<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin,
    C: HandlerConstructor<StreamWrapper<RW>>
{
    stream: Arc<StreamWrapper<RW>>,
    rtmp_context: Arc<RtmpContext>,
    handler_constructor: PhantomData<C>
}

impl<RW, C> Server<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin,
    C: HandlerConstructor<StreamWrapper<RW>>
{
    /// Constructs a Server instance.
    pub fn new(stream: RW, rtmp_context: RtmpContext, handler_constructor: PhantomData<C>) -> Self {
        Self {
            stream: Arc::new(StreamWrapper::new(stream)),
            rtmp_context: Arc::new(rtmp_context),
            handler_constructor
        }
    }
}

impl<RW, C> Future for Server<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin,
    C: HandlerConstructor<StreamWrapper<RW>>
{
    type Output = IOResult<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        pin!(C::new(Arc::clone(&self.stream))).poll_handle(cx, self.rtmp_context.make_weak_mut())
    }
}
