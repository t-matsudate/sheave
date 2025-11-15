use std::{
    future::Future,
    io::Result as IOResult,
    pin::Pin,
    task::{
        Context as FutureContext,
        Poll
    }
};
use tokio::io::AsyncWrite;
use crate::handshake::Handshake;

#[doc(hidden)]
#[derive(Debug)]
pub struct HandshakeWriter<'a, W: AsyncWrite> {
    writer: Pin<&'a mut W>,
    handshake: &'a Handshake
}

#[doc(hidden)]
impl<W: AsyncWrite> Future for HandshakeWriter<'_, W> {
    type Output = IOResult<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        let handshake_bytes = self.handshake.get_bytes();
        self.writer.as_mut().poll_write(cx, handshake_bytes).map_ok(|_| ())
    }
}

/// Writes actual handshake data into streams.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::{
///         Pin,
///         pin
///     },
///     time::Duration
/// };
/// use sheave_core::{
///     handshake::{
///         Handshake,
///         Version
///     },
///     writers::write_handshake
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     let handshake = Handshake::new(Duration::default(), Version::UNSIGNED);
///     write_handshake(writer.as_mut(), &handshake).await?;
///     assert_eq!(handshake.get_bytes(), writer.as_slice());
///     Ok(())
/// }
/// ```
pub fn write_handshake<'a, W: AsyncWrite>(writer: Pin<&'a mut W>, handshake: &'a Handshake) -> HandshakeWriter<'a, W> {
    HandshakeWriter { writer, handshake }
}

#[cfg(test)]
mod tests {
    use std::{
        pin::pin,
        time::Duration
    };
    use crate::handshake::{
        Handshake,
        Version
    };
    use super::*;

    #[tokio::test]
    async fn read_unsigned() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let handshake = Handshake::new(Duration::default(), Version::UNSIGNED);
        let result = write_handshake(writer.as_mut(), &handshake).await;
        assert!(result.is_ok());
        assert_eq!(handshake.get_bytes(), writer.as_slice())
    }
}
