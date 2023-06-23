mod message_format;

pub use self::message_format::MessageFormat;

/// Indicates the chunk stream and message header's format.
/// This header has 3 types.
///
/// |Total Length|Message Header Format|Chunk ID|Chunk ID Range|
/// | ---------: | ------------------: | -----: | -----------: |
/// |8           |2                    |6       |0 - 63        |
/// |16          |2                    |8       |64 - 319      |
/// |24          |2                    |16      |64 - 65599    |
///
/// Unit of every item is bits.
/// Basic header which is and above 16 bits has a flag bits in first 8 bits.
/// It means whether chunk ID is 16 bits.
/// Note if chunk ID is 16 bits, encoding/decoding it as Little Endian is required.
///
/// Any Chunk ID which is and above 64 is required to add/subtract 64 from it when reading/writing.
/// This considers a chunk ID which is replaced with the flag in first 1 byte.
#[derive(Debug, Clone, Copy)]
pub struct BasicHeader {
    message_format: MessageFormat,
    chunk_id: u16
}

impl BasicHeader {
    pub fn new(message_format: MessageFormat, chunk_id: u16) -> Self {
        Self { message_format, chunk_id }
    }

    pub fn get_message_format(&self) -> MessageFormat {
        self.message_format
    }

    pub fn get_chunk_id(&self) -> u16 {
        self.chunk_id
    }
}
