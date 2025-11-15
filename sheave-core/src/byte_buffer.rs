mod insufficient_buffer_length;

use std::io::Result as IOResult;
use crate::U24_MAX;
pub use self::insufficient_buffer_length::*;

/// The stream buffer for encoding/decoding chunk data.
#[derive(Debug, Clone, Default)]
pub struct ByteBuffer {
    bytes: Vec<u8>,
    offset: usize
}

impl ByteBuffer {
    /// Computes remained length in this buffer.
    pub fn remained(&self) -> usize {
        self.bytes.len() - self.offset
    }

    /// Peeks 1 byte from buffer.
    ///
    /// This keeps buffer's current offset.
    ///
    /// # Errors
    ///
    /// * [`InSufficientBufferLength`]
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
    /// * [`InsufficientBufferLength`]
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
    /// * [`InsufficientBufferLength`]
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

    /// Tries getting **signed** 3 bytes from buffer, as the big endian.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When buffer isn't remained at least 2 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::fill;
    /// use sheave_core::ByteBuffer;
    ///
    /// let mut bytes: [u8; 3] = [0; 3];
    /// fill(&mut bytes);
    /// let mut buffer: ByteBuffer = bytes.to_vec().into();
    /// assert!(buffer.get_i24_be().is_ok());
    ///
    /// let mut buffer: ByteBuffer = Vec::new().into();
    /// assert!(buffer.get_i24_be().is_err());
    /// ```
    ///
    /// [`InsufficientBufferLength`]: InsufficientBufferLength
    pub fn get_i24_be(&mut self) -> IOResult<i32> {
        let offset = self.offset;
        self.bytes.get(offset..(offset + 3)).map(
            |bytes| {
                self.offset += bytes.len();
                let mut _bytes: [u8; 4] = [0; 4];
                _bytes[1..].copy_from_slice(bytes);
                let mut n = i32::from_be_bytes(_bytes);

                // Moves the most significant bit if this is a negative number in 3 bytes.
                if (n & 0x00800000) != 0 {
                    n ^= -1;
                    n += 1;
                }

                n
            }
        ).ok_or(insufficient_buffer_length(3, self.remained()))
    }

    /// Tries getting 3 bytes from buffer, as the big endian.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When buffer isn't remained at least 2 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::fill;
    /// use sheave_core::ByteBuffer;
    ///
    /// let mut bytes: [u8; 3] = [0; 3];
    /// fill(&mut bytes);
    /// let mut buffer: ByteBuffer = bytes.to_vec().into();
    /// assert!(buffer.get_u24_be().is_ok());
    ///
    /// let mut buffer: ByteBuffer = Vec::new().into();
    /// assert!(buffer.get_u24_be().is_err());
    /// ```
    ///
    /// [`InsufficientBufferLength`]: InsufficientBufferLength
    pub fn get_u24_be(&mut self) -> IOResult<u32> {
        let offset = self.offset;
        self.bytes.get(offset..(offset + 3)).map(
            |bytes| {
                self.offset += bytes.len();
                let mut _bytes: [u8; 4] = [0; 4];
                _bytes[1..].copy_from_slice(bytes);
                u32::from_be_bytes(_bytes)
            }
        ).ok_or(insufficient_buffer_length(3, self.remained()))
    }

    /// Tries getting 4 bytes from buffer, as the big endian.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When buffer isn't remained at least 4 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::random;
    /// use sheave_core::ByteBuffer;
    ///
    /// let mut buffer: ByteBuffer = random::<u32>().to_be_bytes().to_vec().into();
    /// assert!(buffer.get_u32_be().is_ok());
    ///
    /// let mut buffer: ByteBuffer = Vec::new().into();
    /// assert!(buffer.get_u32_be().is_err());
    /// ```
    ///
    /// [`InsufficientBufferLength`]: InsufficientBufferLength
    pub fn get_u32_be(&mut self) -> IOResult<u32> {
        let offset = self.offset;
        self.bytes.get(offset..(offset + 4)).map(
            |bytes| {
                self.offset += bytes.len();
                let mut _bytes: [u8; 4] = [0; 4];
                _bytes.copy_from_slice(bytes);
                u32::from_be_bytes(_bytes)
            }
        ).ok_or(insufficient_buffer_length(4, self.remained()))
    }

    /// Tries getting 8 bytes from buffer, as a 64 bits floating point number.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
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
    pub fn put_u8(&mut self, n: u8) {
        self.bytes.push(n);
    }

    /// Puts 2 bytes into buffer, as the big endian.
    pub fn put_u16_be(&mut self, n: u16) {
        self.bytes.extend_from_slice(&n.to_be_bytes());
    }

    /// Puts **signed** 3 bytes into buffer, as the big endian.
    ///
    /// # Panics
    ///
    /// Its value must be the range of 24 bits.
    /// If it exceeds, a panic is occured.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::panic::catch_unwind;
    ///
    /// let result = catch_unwind(
    ///     || {
    ///         use sheave_core::ByteBuffer;
    ///
    ///         let mut buffer = ByteBuffer::default();
    ///         buffer.put_i24_be((0x00ffffff + 1) as i32);
    ///     }
    /// );
    /// assert!(result.is_err())
    /// ```
    pub fn put_i24_be(&mut self, n: i32) {
        assert!(n <= U24_MAX as i32);
        self.bytes.extend_from_slice(&n.to_be_bytes()[1..]);
    }

    /// Puts 3 bytes into buffer, as the big endian.
    ///
    /// # Panics
    ///
    /// Its value must be the range of 24 bits.
    /// If it exceeds, a panic is occured.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::panic::catch_unwind;
    ///
    /// let result = catch_unwind(
    ///     || {
    ///         use sheave_core::ByteBuffer;
    ///
    ///         let mut buffer = ByteBuffer::default();
    ///         buffer.put_u24_be((0x00ffffff + 1) as u32);
    ///     }
    /// );
    /// assert!(result.is_err())
    /// ```
    pub fn put_u24_be(&mut self, n: u32) {
        assert!(n <= U24_MAX as u32);
        self.bytes.extend_from_slice(&n.to_be_bytes()[1..]);
    }

    /// Puts 4 bytes into buffer, as the big endian.
    pub fn put_u32_be(&mut self, n: u32) {
        self.bytes.extend_from_slice(&n.to_be_bytes());
    }

    /// Puts 8 bytes into buffer, as a 64 bits floating point number.
    pub fn put_f64(&mut self, n: f64) {
        self.bytes.extend_from_slice(&n.to_be_bytes());
    }

    /// Puts arbitrary bytes into buffer.
    pub fn put_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }
}

impl From<Vec<u8>> for ByteBuffer {
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
    /// use rand::fill;
    /// use sheave_core::ByteBuffer;
    ///
    /// // When all byte is remained.
    /// let mut array: [u8; 128] = [0; 128];
    /// fill(&mut array);
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_bytes(&array);
    /// let bytes: Vec<u8> = buffer.into();
    /// assert_eq!(array.as_slice(), &bytes);
    ///
    /// // When some bytes are consumed.
    /// let mut array: [u8; 128] = [0; 128];
    /// fill(&mut array);
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
