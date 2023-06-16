use super::message_format::MessageFormat;

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
