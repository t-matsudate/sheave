use std::{
    future::Future,
    io::Result as IOResult,
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
    HandlerConstructor,
    RtmpContext,
    StreamWrapper
};

/// # The client instance of the Sheave
///
/// This consists of:
///
/// * Some stream instance which can both of read and write.
/// * Context data in the client.
/// * Some type parameter which implemented the [`HandlerConstructor`] trait.
///
/// The client wraps streams into [`Arc`] as a way of sharing streams among communication steps.
/// And also wraps contexts because of the same purpose.
///
/// The client makes any foreign handler to be able to construct via the [`PhantomData`], where a type parameter of [`PhantomData`] requires to implement the [`HandlerConstructor`] trait.
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
/// use sheave_client::Client;
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
///     let mut client = Client::new(stream, rtmp_context, PhantomData::<SomethingHandler<VecStream>>);
///     let result = client.await;
///     assert!(result.is_ok())
/// }
/// ```
///
/// [`Arc`]: std::sync::Arc
/// [`PhantomData`]: std::marker::PhantomData
/// [`HandlerConstructor`]: sheave_core::handlers::HandlerConstructor
#[derive(Debug)]
pub struct Client<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin,
    C: HandlerConstructor<StreamWrapper<RW>>
{
    stream: Arc<StreamWrapper<RW>>,
    rtmp_context: Arc<RtmpContext>,
    handler_constructor: PhantomData<C>
}

impl<RW, C> Client<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin,
    C: HandlerConstructor<StreamWrapper<RW>>
{
    /// Constructs a Client instance.
    pub fn new(stream: RW, rtmp_context: RtmpContext, handler_constructor: PhantomData<C>) -> Self {
        Self {
            stream: Arc::new(StreamWrapper::new(stream)),
            rtmp_context: Arc::new(rtmp_context),
            handler_constructor
        }
    }
}

impl<RW, C> Future for Client<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin,
    C: HandlerConstructor<StreamWrapper<RW>>
{
    type Output = IOResult<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        pin!(C::new(Arc::clone(&self.stream))).poll_handle(cx, self.rtmp_context.make_weak_mut())
    }
}
