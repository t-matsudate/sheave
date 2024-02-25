/// Representation of message type id byte of the Message header.
///
/// Variants correspond to respectively following numbers:
///
/// |Pattern|Number|
/// | :- | :- |
/// |`ChunkSize`|`1`|
/// |`UserControl`|`4`|
/// |`Data`|`18`|
/// |`Command`|`20`|
/// |`Other`|other numbers|
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    ChunkSize = 1,
    UserControl = 4,
    Data = 18,
    Command = 20,
    Other = 0xff
}

impl From<u8> for MessageType {
    fn from(message_type: u8) -> Self {
        use MessageType::*;

        match message_type {
            1 => ChunkSize,
            4 => UserControl,
            18 => Data,
            20 => Command,
            _ => Other
        }
    }
}

impl From<MessageType> for u8 {
    fn from(message_type: MessageType) -> Self {
        message_type as u8
    }
}
