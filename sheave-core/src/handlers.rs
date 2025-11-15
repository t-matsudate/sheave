mod rtmp_context;
mod inconsistent_sha;
mod stream_wrapper;
mod vec_stream;
mod status;
mod measure_acknowledgement;
mod chain;
mod while_ok;
mod middlewares;
mod map_err;
mod stream_got_exhausted;
mod client_type;

use std::{
    io::Result as IOResult,
    pin::Pin,
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
use self::{
    chain::*,
    while_ok::*,
    middlewares::{
        Wrap,
        wrap
    },
    map_err::{
        MapErr,
        map_err,
    }
};
pub use self::{
    rtmp_context::*,
    inconsistent_sha::*,
    stream_wrapper::*,
    vec_stream::*,
    status::*,
    middlewares::Middleware,
    map_err::ErrorHandler,
    measure_acknowledgement::*,
    stream_got_exhausted::*,
    client_type::*
};

/// The interface for handling RTMP connection steps with `Future`.
///
/// This trait unifies surfaces of handler APIs:
///
/// * `RtmpContext` is required.
/// * Terminating with unit (`()`) is required.
///
/// The first requirement makes `RtmpContext` reusable for upper APIs.
/// And the second requirement makes handlers return `Ok(())` when successfully terminates because currently they are run on `main`.
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::Pin,
///     sync::Arc,
///     task::{
///         Context as FutureContext,
///         Poll
///     }
/// };
/// use futures::future::poll_fn;
/// use tokio::io::{
///     AsyncRead,
///     AsyncWrite
/// };
/// use sheave_core::handlers::{
///     AsyncHandler,
///     RtmpContext
/// };
///
/// struct SomethingHandler<RW: AsyncRead + AsyncWrite + Unpin>(Arc<RW>);
///
/// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for SomethingHandler<RW> {
///     fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
///         // Something to handle
///
///         Poll::Ready(Ok(()))
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     // Consider this is Tokio's `JoinHandle` which is run on `main`.
///     poll_fn(
///         |cx| {
///             use std::{
///                 pin::pin,
///                 sync::Arc
///             };
///             use sheave_core::handlers::{
///                 AsyncHandler,
///                 VecStream,
///                 StreamWrapper
///             };
///
///             let stream = Arc::new(StreamWrapper::new(VecStream::default()));
///             pin!(SomethingHandler(stream)).poll_handle(cx, &mut RtmpContext::default())
///         }
///     ).await
/// }
/// ```
///
/// [`RtmpContext`]: RtmpContext
pub trait AsyncHandler {
    fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>>;
}

/// The extension methods for handlers.
///
/// Currently following extensions have been implemented.
///
/// * [`chain`]
/// * [`wrap`]
/// * [`while_ok`]
/// * [`map_err`]
///
/// [`chain`]: AsyncHandlerExt::chain
/// [`wrap`]: AsyncHandlerExt::wrap
/// [`while_ok`]: AsyncHandlerExt::while_ok
/// [`map_err`]: AsyncHandlerExt::map_err
pub trait AsyncHandlerExt: AsyncHandler {
    /// Chains this handler with `next`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::{
    ///     io::Result as IOResult,
    ///     pin::Pin,
    ///     sync::Arc,
    ///     task::{
    ///         Context as FutureContext,
    ///         Poll
    ///     }
    /// };
    /// use futures::future::poll_fn;
    /// use tokio::io::{
    ///     AsyncRead,
    ///     AsyncWrite
    /// };
    /// use sheave_core::handlers::{
    ///     AsyncHandler,
    ///     RtmpContext
    /// };
    ///
    /// struct HandlerA<RW: AsyncRead + AsyncWrite + Unpin>(Arc<RW>);
    /// struct HandlerB<RW: AsyncRead + AsyncWrite + Unpin>(Arc<RW>);
    ///
    /// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for HandlerA<RW> {
    ///     fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
    ///         // Something to handle.
    ///
    ///         Poll::Ready(Ok(()))
    ///     }
    /// }
    ///
    /// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for HandlerB<RW> {
    ///     fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
    ///         // Something to handle.
    ///
    ///         Poll::Ready(Ok(()))
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> IOResult<()> {
    ///     poll_fn(
    ///         |cx| {
    ///             use std::pin::pin;
    ///             use sheave_core::handlers::{
    ///                 AsyncHandlerExt,
    ///                 StreamWrapper,
    ///                 VecStream
    ///             };
    ///
    ///             let stream = Arc::new(StreamWrapper::new(VecStream::default()));
    ///             pin!(
    ///                 HandlerA(Arc::clone(&stream))
    ///                     .chain(HandlerB(Arc::clone(&stream)))
    ///             ).poll_handle(cx, &mut RtmpContext::default())
    ///         }
    ///     ).await
    /// }
    /// ```
    fn chain<H>(self, next: H) -> Chain<Self, H>
    where
        H: AsyncHandler + Unpin,
        Self: Sized + Unpin
    {
        chain(self, next)
    }

    /// Wraps previous handlers into a middleware.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::{
    ///     io::Result as IOResult,
    ///     pin::Pin,
    ///     sync::Arc,
    ///     task::{
    ///         Context as FutureContext,
    ///         Poll
    ///     }
    /// };
    /// use futures::{
    ///     future::poll_fn,
    ///     ready
    /// };
    /// use tokio::io::{
    ///     AsyncRead,
    ///     AsyncWrite
    /// };
    /// use sheave_core::handlers::{
    ///     AsyncHandler,
    ///     Middleware,
    ///     RtmpContext
    /// };
    ///
    /// struct SomethingHandler<RW: AsyncRead + AsyncWrite + Unpin>(Arc<RW>);
    ///
    /// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for SomethingHandler<RW> {
    ///     fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
    ///         // Something to handle.
    ///
    ///         Poll::Ready(Ok(()))
    ///     }
    /// }
    ///
    /// struct SomethingMiddleware<'a, W: Unpin>(Pin<&'a mut W>);
    ///
    /// impl<W: Unpin> Middleware for SomethingMiddleware<'_, W> {
    ///     fn poll_handle_wrapped<H: AsyncHandler + Unpin>(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext, handler: Pin<&mut H>) -> Poll<IOResult<()>> {
    ///         println!("Starts wrapping.");
    ///         ready!(handler.poll_handle(cx, rtmp_context))?;
    ///         println!("Ends wrapping.");
    ///         Poll::Ready(Ok(()))
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let result = poll_fn(
    ///         |cx| {
    ///             use std::pin::pin;
    ///             use sheave_core::handlers::{
    ///                 AsyncHandlerExt,
    ///                 StreamWrapper,
    ///                 VecStream
    ///             };
    ///
    ///             let stream = Arc::new(StreamWrapper::new(VecStream::default()));
    ///             pin!(
    ///                 SomethingHandler(Arc::clone(&stream))
    ///                     .wrap(SomethingMiddleware(stream.make_weak_pin()))
    ///             ).poll_handle(cx, &mut RtmpContext::default())
    ///         }
    ///     ).await;
    ///     assert!(result.is_ok())
    /// }
    /// ```
    fn wrap<M>(self, middleware: M) -> Wrap<M, Self>
    where
        M: Middleware + Unpin,
        Self: Sized + Unpin
    {
        wrap(middleware, self)
    }

    /// Loops while the body returns `Ok(())` or `Pending`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::{
    ///     io::{
    ///         Error as IOError,
    ///         ErrorKind,
    ///         Result as IOResult
    ///     },
    ///     pin::Pin,
    ///     sync::Arc,
    ///     task::{
    ///         Context as FutureContext,
    ///         Poll
    ///     }
    /// };
    /// use futures::future::poll_fn;
    /// use tokio::io::{
    ///     AsyncRead,
    ///     AsyncWrite
    /// };
    /// use sheave_core::handlers::{
    ///     AsyncHandler,
    ///     RtmpContext,
    ///     StreamWrapper
    /// };
    ///
    /// struct SomethingHandler<RW: AsyncRead + AsyncWrite + Unpin>(Arc<RW>);
    ///
    /// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for SomethingHandler<RW> {
    ///     fn poll_handle(self: Pin<&mut Self>, _cx: &mut FutureContext<'_>, _rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
    ///         // Something to handle.
    ///
    ///         Poll::Ready(Ok(()))
    ///     }
    /// }
    ///
    /// struct AnotherHandler<RW: AsyncRead + AsyncWrite + Unpin>(Arc<RW>);
    ///
    /// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for AnotherHandler<RW> {
    ///     fn poll_handle(self: Pin<&mut Self>, _cx: &mut FutureContext<'_>, _rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
    ///         Poll::Ready(Err(IOError::from(ErrorKind::Other)))
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let result = poll_fn(
    ///         |cx| {
    ///             use std::pin::pin;
    ///             use sheave_core::handlers::{
    ///                 AsyncHandlerExt,
    ///                 VecStream
    ///             };
    ///
    ///             let stream = Arc::new(StreamWrapper::new(VecStream::default()));
    ///             pin!(
    ///                 SomethingHandler(Arc::clone(&stream))
    ///                     .while_ok(AnotherHandler(Arc::clone(&stream)))
    ///             ).poll_handle(cx, &mut RtmpContext::default())
    ///         }
    ///     ).await;
    ///     assert!(result.is_err())
    /// }
    /// ```
    fn while_ok<H>(self, body: H) -> WhileOk<Self, H>
    where
        H: AsyncHandler + Unpin,
        Self: Sized + Unpin
    {
        while_ok(self, body)
    }

    /// Handles some error when previous handler returns `Err`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::{
    ///     io::{
    ///         Error as IOError,
    ///         Result as IOResult
    ///     },
    ///     pin::Pin,
    ///     sync::Arc,
    ///     task::{
    ///         Context as FutureContext,
    ///         Poll
    ///     }
    /// };
    /// use futures::future::poll_fn;
    /// use tokio::io::{
    ///     AsyncRead,
    ///     AsyncWrite
    /// };
    /// use sheave_core::handlers::{
    ///     AsyncHandler,
    ///     ErrorHandler,
    ///     RtmpContext
    /// };
    ///
    /// struct SomethingHandler<RW: AsyncRead + AsyncWrite + Unpin>(Arc<RW>);
    ///
    /// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for SomethingHandler<RW> {
    ///     fn poll_handle(self: Pin<&mut Self>, _cx: &mut FutureContext<'_>, _rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
    ///         Poll::Ready(Err(IOError::other("Something Wrong.")))
    ///     }
    /// }
    ///
    /// struct SomethingWrongHandler<'a, RW>(Pin<&'a mut RW>);
    ///
    /// impl<RW> ErrorHandler for SomethingWrongHandler<'_, RW> {
    ///     fn poll_handle_error(self: Pin<&mut Self>, _cx: &mut FutureContext<'_>, _rtmp_context: &mut RtmpContext, error: IOError) -> Poll<IOResult<()>> {
    ///         println!("{error}");
    ///
    ///         // This `Ok` means that handled its error successfully.
    ///         Poll::Ready(Ok(()))
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let result = poll_fn(
    ///         |cx| {
    ///             use std::pin::pin;
    ///             use sheave_core::handlers::{
    ///                 AsyncHandlerExt,
    ///                 StreamWrapper,
    ///                 VecStream
    ///             };
    ///
    ///             let stream = Arc::new(StreamWrapper::new(VecStream::default()));
    ///             pin!(
    ///                 SomethingHandler(Arc::clone(&stream))
    ///                     .map_err(SomethingWrongHandler(stream.make_weak_pin()))
    ///             ).poll_handle(cx, &mut RtmpContext::default())
    ///         }
    ///     ).await;
    ///     assert!(result.is_ok())
    /// }
    /// ```
    fn map_err<E>(self, error_handler: E) -> MapErr<Self, E>
    where
        E: ErrorHandler + Unpin,
        Self: Sized + Unpin
    {
        map_err(self, error_handler)
    }
}

impl<H: AsyncHandler> AsyncHandlerExt for H {}

/// The interface for providing the way to construct any handler to clients/servers.
///
/// Servers / Clients pass streams and contexts to any handler they contain.
/// Here we are necessary to be careful that some stream can't clone. (e.g. sockets)
/// But we need to share these while handling RTMP communication steps.
/// Therefore this provides the way of cloning stream instances via the (smart) pointer.
///
/// # Examples
///
/// ```rust
/// use std::{
///     future::Future,
///     io::Result as IOResult,
///     marker::PhantomData,
///     pin::{
///         Pin,
///         pin
///     },
///     sync::Arc,
///     task::{
///         Context as FutureContext,
///         Poll
///     }
/// };
/// use tokio::io::{
///     AsyncRead,
///     AsyncWrite,
///     ReadBuf
/// };
/// use sheave_core::handlers::{
///     AsyncHandler,
///     HandlerConstructor,
///     RtmpContext
/// };
///
/// struct SomethingStream;
///
/// impl AsyncRead for SomethingStream {
///     fn poll_read(self: Pin<&mut Self>, _cx: &mut FutureContext<'_>, _buf: &mut ReadBuf<'_>) -> Poll<IOResult<()>> {
///         // Something to read.
///
///         Poll::Ready(Ok(()))
///     }
/// }
///
/// impl AsyncWrite for SomethingStream {
///     fn poll_write(self: Pin<&mut Self>, _cx: &mut FutureContext<'_>, buf: &[u8]) -> Poll<IOResult<usize>> {
///         // Something to write.
///
///         Poll::Ready(Ok(buf.len()))
///     }
///
///     fn poll_flush(self: Pin<&mut Self>, _cx: &mut FutureContext<'_>) -> Poll<IOResult<()>> {
///         // Something to flush.
///
///         Poll::Ready(Ok(()))
///     }
///
///     fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut FutureContext<'_>) -> Poll<IOResult<()>> {
///         // Something to shutdown.
///
///         Poll::Ready(Ok(()))
///     }
/// }
///
/// struct SomethingHandler<RW: AsyncRead + AsyncWrite + Unpin>(Arc<RW>);
///
/// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for SomethingHandler<RW> {
///     fn poll_handle(self: Pin<&mut Self>, _cx: &mut FutureContext<'_>, _rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
///         // Something to handle.
///
///         Poll::Ready(Ok(()))
///     }
/// }
///
/// impl<RW: AsyncRead + AsyncWrite + Unpin> HandlerConstructor<RW> for SomethingHandler<RW> {
///     fn new(stream: Arc<RW>) -> Self {
///         Self(stream)
///     }
/// }
///
/// struct SomethingRunner<RW, C>
/// where
///     RW: AsyncRead + AsyncWrite + Unpin,
///     C: HandlerConstructor<RW>
/// {
///     stream: Arc<RW>,
///     rtmp_context: Arc<RtmpContext>,
///     handler_constructor: PhantomData<C>
/// }
///
/// impl<RW, C> SomethingRunner<RW, C>
/// where
///     RW: AsyncRead + AsyncWrite + Unpin,
///     C: HandlerConstructor<RW>
/// {
///     pub fn new(stream: RW, rtmp_context: RtmpContext, handler_constructor: PhantomData<C>) -> Self {
///         Self {
///             stream: Arc::new(stream),
///             rtmp_context: Arc::new(rtmp_context),
///             handler_constructor
///         }
///     }
/// }
///
/// impl<RW, C> Future for SomethingRunner<RW, C>
/// where
///     RW: AsyncRead + AsyncWrite + Unpin,
///     C: HandlerConstructor<RW>
/// {
///     type Output = IOResult<()>;
///
///     fn poll(self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
///         pin!(C::new(Arc::clone(&self.stream))).poll_handle(cx, self.rtmp_context.make_weak_mut())
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let stream = SomethingStream;
///     let rtmp_context = RtmpContext::default();
///     let handler_constructor = PhantomData::<SomethingHandler<SomethingStream>>;
///     let runner = SomethingRunner::new(stream, rtmp_context, handler_constructor);
///     let result = runner.await;
///
///     assert!(result.is_ok());
/// }
/// ```
pub trait HandlerConstructor<RW: AsyncRead + AsyncWrite + Unpin>: AsyncHandler {
    fn new(stream: Arc<RW>) -> Self;
}
