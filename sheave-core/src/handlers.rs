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
mod chain;

use std::{
    io::Result as IOResult,
    pin::Pin,
    task::{
        Context as FutureContext,
        Poll
    }
};
use self::chain::*;
pub use self::{
    rtmp_context::*,
    inconsistent_sha::*,
    stream_wrapper::*,
    vec_stream::*
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
}

impl<H: AsyncHandler> AsyncHandlerExt for H {}
