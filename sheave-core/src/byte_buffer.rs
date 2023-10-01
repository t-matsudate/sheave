mod insufficient_buffer_length;

use std::io::Result as IOResult;
pub use self::insufficient_buffer_length::*;

/// The stream buffer for encoding/decoding chunk data.
#[derive(Debug, Clone, Default)]
pub struct ByteBuffer {
    bytes: Vec<u8>,
    offset: usize
}

impl ByteBuffer {
    /// Computes remained length in this buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::{
    ///     Fill,
    ///     thread_rng
    /// };
    /// use sheave_core::ByteBuffer;
    ///
    /// let mut bytes: [u8; 128] = [0; 128];
    /// bytes.try_fill(&mut thread_rng()).unwrap();
    /// let mut buffer: ByteBuffer = bytes.to_vec().into();
    /// assert_eq!(bytes.len(), buffer.remained());
    /// buffer.get_u8().unwrap();
    /// assert_eq!(bytes[1..].len(), buffer.remained())
    /// ```
    pub fn remained(&self) -> usize {
        self.bytes.len() - self.offset
    }

    /// Peeks 1 byte from buffer.
    /// This keeps buffer's current offset.
    ///
    /// # Errors
    ///
    /// * `InSufficientBufferLength`
    ///
    /// When buffer isn't remained at least 1 byte.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::random;
    /// use sheave_core::ByteBuffer;
    ///
    /// let n = random::<u8>();
    /// let mut buffer: ByteBuffer = n.to_be_bytes().to_vec().into();
    /// assert!(buffer.peek_u8().is_ok());
    /// let bytes: Vec<u8> = buffer.into();
    /// // Buffer isn't consumed.
    /// assert_eq!(n.to_be_bytes().as_slice(), &bytes);
    ///
    /// let mut buffer: ByteBuffer = Vec::new().into();
    /// assert!(buffer.peek_u8().is_err());
    /// ```
    ///
    /// [`InsufficientBufferLength`]: InsufficientBufferLength
    pub fn peek_u8(&self) -> IOResult<u8> {
        let offset = self.offset;
        self.bytes.get(offset).map(|byte| *byte).ok_or(insufficient_buffer_length(1, self.remained()))
    }

    /// Tries getting 1 byte from buffer.
    ///
    /// # Errors
    ///
    /// * `InsufficientBufferLength`
    ///
    /// When buffer isn't remained at least 1 byte.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::random;
    /// use sheave_core::ByteBuffer;
    ///
    /// let mut buffer: ByteBuffer  = random::<u8>().to_be_bytes().to_vec().into();
    /// assert!(buffer.get_u8().is_ok());
    ///
    /// let mut buffer: ByteBuffer = Vec::new().into();
    /// assert!(buffer.get_u8().is_err());
    /// ```
    ///
    /// [`InsufficientBufferLength`]: InsufficientBufferLength
    pub fn get_u8(&mut self) -> IOResult<u8> {
        let offset = self.offset;
        self.bytes.get(offset).map(
            |byte| {
                self.offset += 1;
                *byte
            }
        ).ok_or(insufficient_buffer_length(1, self.remained()))
    }

    /// Tries getting 2 bytes from buffer, as the big endian.
    ///
    /// # Errors
    ///
    /// * `InsufficientBufferLength`
    ///
    /// When buffer isn't remained at least 2 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::random;
    /// use sheave_core::ByteBuffer;
    ///
    /// let mut buffer: ByteBuffer = random::<u16>().to_be_bytes().to_vec().into();
    /// assert!(buffer.get_u16_be().is_ok());
    ///
    /// let mut buffer: ByteBuffer = Vec::new().into();
    /// assert!(buffer.get_u16_be().is_err());
    /// ```
    ///
    /// [`InsufficientBufferLength`]: InsufficientBufferLength
    pub fn get_u16_be(&mut self) -> IOResult<u16> {
        let offset = self.offset;
        self.bytes.get(offset..(offset + 2)).map(
            |bytes| {
                self.offset += bytes.len();
                let mut _bytes: [u8; 2] = [0; 2];
                _bytes.copy_from_slice(bytes);
                u16::from_be_bytes(_bytes)
            }
        ).ok_or(insufficient_buffer_length(2, self.remained()))
    }

    /// Tries getting 8 bytes from buffer, as a 64 bits floating point number.
    ///
    /// # Errors
    ///
    /// * `InsufficientBufferLength`
    ///
    /// When buffer isn't remained at least 8 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::random;
    /// use sheave_core::ByteBuffer;
    ///
    /// let mut buffer: ByteBuffer = random::<u64>().to_be_bytes().to_vec().into();
    /// assert!(buffer.get_f64().is_ok());
    ///
    /// let mut buffer: ByteBuffer = Vec::new().into();
    /// assert!(buffer.get_f64().is_err());
    /// ```
    ///
    /// [`InsufficientBufferLength`]: InsufficientBufferLength
    pub fn get_f64(&mut self) -> IOResult<f64> {
        let offset = self.offset;
        self.bytes.get(offset..(offset + 8)).map(
            |bytes| {
                self.offset += bytes.len();
                let mut _bytes: [u8; 8] = [0; 8];
                _bytes.copy_from_slice(bytes);
                f64::from_be_bytes(_bytes)
            }
        ).ok_or(insufficient_buffer_length(8, self.remained()))
    }

    /// Tries getting arbitrary bytes from buffer.
    ///
    /// # Erorrs
    ///
    /// * `InsufficientBufferLength`
    ///
    /// When buffer isn't remained at least specified length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::ByteBuffer;
    ///
    /// let bytes = "hello world!".as_bytes();
    /// let mut buffer: ByteBuffer = bytes.to_vec().into();
    /// assert!(buffer.get_bytes(bytes.len()).is_ok());
    ///
    /// let mut buffer: ByteBuffer = Vec::new().into();
    /// assert!(buffer.get_bytes(bytes.len()).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: InsufficientBufferLength
    pub fn get_bytes(&mut self, len: usize) -> IOResult<&[u8]> {
        let offset = self.offset;
        self.bytes.get(offset..(offset + len)).map(
            |bytes| {
                self.offset += len;
                bytes
            }
        ).ok_or(insufficient_buffer_length(len, self.remained()))
    }

    /// Puts 1 byte into buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::random;
    /// use sheave_core::ByteBuffer;
    ///
    /// let n = random::<u8>();
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(n);
    /// assert_eq!(n, buffer.get_u8().unwrap())
    /// ```
    pub fn put_u8(&mut self, n: u8) {
        self.bytes.push(n);
    }

    /// Puts 2 bytes into buffer, as the big endian.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::random;
    /// use sheave_core::ByteBuffer;
    ///
    /// let n = random::<u16>();
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u16_be(n);
    /// assert_eq!(n, buffer.get_u16_be().unwrap())
    /// ```
    pub fn put_u16_be(&mut self, n: u16) {
        self.bytes.extend_from_slice(&n.to_be_bytes());
    }

    /// Puts 8 bytes into buffer, as a 64 bits floating point number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::random;
    /// use sheave_core::ByteBuffer;
    ///
    /// let mut n = f64::from_bits(random::<u64>());
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_f64(n);
    /// assert_eq!(n, buffer.get_f64().unwrap())
    /// ```
    pub fn put_f64(&mut self, n: f64) {
        self.bytes.extend_from_slice(&n.to_be_bytes());
    }

    /// Puts arbitrary bytes into buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::ByteBuffer;
    ///
    /// let s = "hello world!".as_bytes();
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_bytes(s);
    /// assert_eq!(s, buffer.get_bytes(s.len()).unwrap())
    /// ```
    pub fn put_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }
}

impl From<Vec<u8>> for ByteBuffer {
    /// Converts Vec into ByteBuffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::{
    ///     Fill,
    ///     thread_rng
    /// };
    /// use sheave_core::ByteBuffer;
    ///
    /// let mut array: [u8; 128] = [0; 128];
    /// array.try_fill(&mut thread_rng()).unwrap();
    /// let mut buffer: ByteBuffer = array.to_vec().into();
    /// assert_eq!(array.as_slice(), buffer.get_bytes(array.len()).unwrap())
    /// ```
    fn from(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            offset: 0
        }
    }
}

impl From<ByteBuffer> for Vec<u8> {
    /// Converts remained bytes into Vec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::{
    ///     Fill,
    ///     thread_rng
    /// };
    /// use sheave_core::ByteBuffer;
    ///
    /// // When all byte is remained.
    /// let mut array: [u8; 128] = [0; 128];
    /// array.try_fill(&mut thread_rng()).unwrap();
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_bytes(&array);
    /// let bytes: Vec<u8> = buffer.into();
    /// assert_eq!(array.as_slice(), &bytes);
    ///
    /// // When some bytes are consumed.
    /// let mut array: [u8; 128] = [0; 128];
    /// array.try_fill(&mut thread_rng()).unwrap();
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_bytes(&array);
    /// buffer.get_u8().unwrap();
    /// let bytes: Vec<u8> = buffer.into();
    /// assert_eq!(&array[1..], &bytes)
    /// ```
    fn from(buffer: ByteBuffer) -> Self {
        buffer.bytes[buffer.offset..].to_vec()
    }
}
