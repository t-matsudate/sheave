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
use crate::handshake::EncryptionAlgorithm;

#[doc(hidden)]
#[derive(Debug)]
pub struct EncryptionAlgorithmWriter<'a, W: AsyncWrite> {
    writer: Pin<&'a mut W>,
    encryption_algorithm: EncryptionAlgorithm
}

#[doc(hidden)]
impl<W: AsyncWrite> Future for EncryptionAlgorithmWriter<'_, W> {
    type Output = IOResult<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
        let encryption_algorithm_byte: [u8; 1] = (self.encryption_algorithm as u8).to_be_bytes();
        self.writer.as_mut().poll_write(cx, encryption_algorithm_byte.as_slice()).map_ok(|_| ())
    }
}

/// Writes one byte to indicate the encryption algorithm into streams.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::{
///         Pin,
///         pin
///     }
/// };
/// use sheave_core::{
///     handshake::EncryptionAlgorithm::*,
///     writers::write_encryption_algorithm
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
///     write_encryption_algorithm(writer.as_mut(), NotEncrypted).await?;
///     assert_eq!(3, writer[0]);
///     Ok(())
/// }
/// ```
pub fn write_encryption_algorithm<W: AsyncWrite>(writer: Pin<&mut W>, encryption_algorithm: EncryptionAlgorithm) -> EncryptionAlgorithmWriter<'_, W> {
    EncryptionAlgorithmWriter { writer, encryption_algorithm }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use crate::handshake::EncryptionAlgorithm::*;
    use super::*;

    #[tokio::test]
    async fn write_not_encrypted() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let result = write_encryption_algorithm(writer.as_mut(), NotEncrypted).await;
        assert!(result.is_ok());
        assert_eq!(3, writer[0])
    }

    #[tokio::test]
    async fn write_diffie_hellman() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let result = write_encryption_algorithm(writer.as_mut(), DiffieHellman).await;
        assert!(result.is_ok());
        assert_eq!(6, writer[0])
    }

    #[tokio::test]
    async fn write_xtea() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let result = write_encryption_algorithm(writer.as_mut(), Xtea).await;
        assert!(result.is_ok());
        assert_eq!(8, writer[0])
    }

    #[tokio::test]
    async fn write_blowfish() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let result = write_encryption_algorithm(writer.as_mut(), Blowfish).await;
        assert!(result.is_ok());
        assert_eq!(9, writer[0])
    }

    #[tokio::test]
    async fn write_other() {
        let mut writer: Pin<&mut Vec<u8>> = pin!(Vec::new());
        let result = write_encryption_algorithm(writer.as_mut(), Other).await;
        assert!(result.is_ok());
        assert_eq!(u8::MAX, writer[0])
    }
}
