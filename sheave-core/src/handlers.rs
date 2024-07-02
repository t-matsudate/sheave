//! # Handling RTMP connections and data streaming.
//!
//! Currently following handlers have been implemented.
//!
//! * [`Handshake`]
//! * [`connect`]
//! * [`releaseStream`]
//! * [`FCPublish`]
//! * [`createStream`]
//! * [`publish`]
//!
//! [`Handshake`]: crate::handshake::Handshake
//! [`connect`]: crate::messages::Connect
//! [`releaseStream`]: crate::messages::ReleaseStream
//! [`FCPublish`]: crate::messages::FcPublish
//! [`createStream`]: crate::messages::CreateStream
//! [`publish`]: crate::messages::Publish

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
///     future::Future,
///     pin::Pin,
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
/// struct SomethingHandler<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);
///
/// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for SomethingHandler<'_, RW> {
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
///             pin!(SomethingHandler(stream.make_weak_pin())).poll_handle(cx, &mut RtmpContext::default())
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
/// * `chain`
///
/// [`chain`]: AsyncHandlerExt::chain
pub trait AsyncHandlerExt: AsyncHandler {
    /// Chains this handler with `next`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::{
    ///     io::Result as IOResult,
    ///     pin::Pin,
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
    /// struct HandlerA<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);
    /// struct HandlerB<'a, RW: AsyncRead + AsyncWrite + Unpin>(Pin<&'a mut RW>);
    ///
    /// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for HandlerA<'_, RW> {
    ///     fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
    ///         // Something to handle.
    ///         Poll::Ready(Ok(()))
    ///     }
    /// }
    ///
    /// impl<RW: AsyncRead + AsyncWrite + Unpin> AsyncHandler for HandlerB<'_, RW> {
    ///     fn poll_handle(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, rtmp_context: &mut RtmpContext) -> Poll<IOResult<()>> {
    ///         // Something to handle.
    ///         Poll::Ready(Ok(()))
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> IOResult<()> {
    ///     poll_fn(
    ///         |cx| {
    ///             use std::{
    ///                 pin::pin,
    ///                 sync::Arc
    ///             };
    ///             use sheave_core::handlers::{
    ///                 AsyncHandler,
    ///                 AsyncHandlerExt,
    ///                 StreamWrapper,
    ///                 VecStream
    ///             };
    ///
    ///             let stream = Arc::new(StreamWrapper::new(VecStream::default()));
    ///             pin!(
    ///                 HandlerA(stream.make_weak_pin())
    ///                     .chain(HandlerB(stream.make_weak_pin()))
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

    fn wrap<M>(self, middleware: M) -> Wrap<M, Self>
    where
        M: Middleware + Unpin,
        Self: Sized + Unpin
    {
        wrap(middleware, self)
    }

    fn while_ok<H>(self, condition: H) -> WhileOk<Self, H>
    where
        H: AsyncHandler + Unpin,
        Self: Sized + Unpin
    {
        while_ok(self, condition)
    }

    fn map_err<E>(self, error_handler: E) -> MapErr<Self, E>
    where
        E: ErrorHandler + Unpin,
        Self: Sized + Unpin
    {
        map_err(self, error_handler)
    }
}

impl<H: AsyncHandler> AsyncHandlerExt for H {}

pub trait HandlerConstructor<RW: AsyncRead + AsyncWrite + Unpin>: AsyncHandler {
    fn new(stream: Arc<RW>) -> Self;
}
