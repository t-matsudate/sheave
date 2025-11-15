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
use crate::handshake::EncryptionAlgorithm;

#[doc(hidden)]
#[derive(Debug)]
pub struct EncryptionAlgorithmReader<'a, R: AsyncRead> {
    reader: Pin<&'a mut R>
}

#[doc(hidden)]
impl<R: AsyncRead> Future for EncryptionAlgorithmReader<'_, R> {
    type Output = IOResult<EncryptionAlgorithm>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        let mut encryption_algorithm_byte: [u8; 1] = [0; 1];
        let mut buf = ReadBuf::new(&mut encryption_algorithm_byte);
        ready!(self.reader.as_mut().poll_read(cx, &mut buf))?;
        Poll::Ready(Ok(u8::from_be_bytes(encryption_algorithm_byte).into()))
    }
}

/// Reads one byte to indicate the encryption algorithm from streams.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::pin
/// };
/// use sheave_core::{
///     handshake::EncryptionAlgorithm::*,
///     readers::read_encryption_algorithm
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let reader: [u8; 1] = [3; 1];
///     let result = read_encryption_algorithm(pin!(reader.as_slice())).await?;
///     assert_eq!(NotEncrypted, result);
///     Ok(())
/// }
/// ```
pub fn read_encryption_algorithm<R: AsyncRead>(reader: Pin<&mut R>) -> EncryptionAlgorithmReader<'_, R> {
    EncryptionAlgorithmReader { reader }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use crate::handshake::EncryptionAlgorithm::*;
    use super::*;

    #[tokio::test]
    async fn read_not_encrypted() {
        let reader: [u8; 1] = [3];
        let result = read_encryption_algorithm(pin!(reader.as_slice())).await;
        assert!(result.is_ok());
        assert_eq!(NotEncrypted, result.unwrap())
    }

    #[tokio::test]
    async fn read_diffie_hellman() {
        let reader: [u8; 1] = [6];
        let result = read_encryption_algorithm(pin!(reader.as_slice())).await;
        assert!(result.is_ok());
        assert_eq!(DiffieHellman, result.unwrap())
    }

    #[tokio::test]
    async fn read_xtea() {
        let reader: [u8; 1] = [8];
        let result = read_encryption_algorithm(pin!(reader.as_slice())).await;
        assert!(result.is_ok());
        assert_eq!(Xtea, result.unwrap())
    }

    #[tokio::test]
    async fn read_blowfish() {
        let reader: [u8; 1] = [9];
        let result = read_encryption_algorithm(pin!(reader.as_slice())).await;
        assert!(result.is_ok());
        assert_eq!(Blowfish, result.unwrap())
    }

    #[tokio::test]
    async fn read_other() {
        let reader: [u8; 1] = [0];
        let result = read_encryption_algorithm(pin!(reader.as_slice())).await;
        assert!(result.is_ok());
        assert_eq!(Other, result.unwrap())
    }
}
