use std::{
    future::Future,
    io::Result as IOResult,
    pin::Pin,
    task::{
        Context as FutureContext,
        Poll
    }
};
use futures::ready;
use tokio::io::{
    AsyncRead,
    ReadBuf
};
use crate::handshake::Handshake;

#[doc(hidden)]
#[derive(Debug)]
pub struct HandshakeReader<'a, R: AsyncRead> {
    reader: Pin<&'a mut R>
}

#[doc(hidden)]
impl<R: AsyncRead> Future for HandshakeReader<'_, R> {
    type Output = IOResult<Handshake>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        let mut handshake_bytes: [u8; 1536] = [0; 1536];
        let mut buf = ReadBuf::new(&mut handshake_bytes);
        ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
        Poll::Ready(Ok(handshake_bytes.into()))
    }
}

/// Reads an actual handshake data from streams.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::pin,
///     time::Duration,
/// };
/// use rand::fill;
/// use sheave_core::{
///     handshake::Version,
///     readers::read_handshake
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let mut reader: [u8; 1536] = [0; 1536];
///     reader[..4].copy_from_slice((Duration::default().as_millis() as u32).to_be_bytes().as_slice());
///     reader[4..8].copy_from_slice(<[u8; 4]>::from(Version::UNSIGNED).as_slice());
///     fill(&mut reader[8..]);
///     let compared = reader;
///     let handshake = read_handshake(pin!(reader.as_slice())).await?;
///     assert_eq!(compared.as_slice(), handshake.get_bytes());
///     Ok(())
/// }
/// ```
pub fn read_handshake<R: AsyncRead>(reader: Pin<&mut R>) -> HandshakeReader<'_, R> {
    HandshakeReader { reader }
}

#[cfg(test)]
mod tests {
    use std::{
        pin::pin,
        time::Duration
    };
    use rand::fill;
    use crate::handshake::Version;
    use super::*;

    #[tokio::test]
    async fn read_handshake_bytes() {
        let mut handshake: [u8; 1536] = [0; 1536];
        handshake[..4].copy_from_slice((Duration::default().as_millis() as u32).to_be_bytes().as_slice());
        handshake[4..8].copy_from_slice(<[u8; 4]>::from(Version::UNSIGNED).as_slice());
        fill(&mut handshake[8..]);
        let compared = handshake;
        let result = read_handshake(pin!(handshake.as_slice())).await;
        assert!(result.is_ok());
        assert_eq!(compared.as_slice(), result.unwrap().get_bytes())
    }
}
