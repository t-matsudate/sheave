/// Representation of message type id byte of the Message header.
///
/// Variants correspond to respectively following numbers:
///
/// |Pattern|Number|
/// | :- | :- |
/// |`ChunkSize`|`1`|
/// |`Acknowledgement`|`3`|
/// |`UserControl`|`4`|
/// |`WindowAcknowledgementSize`|`5`|
/// |`PeerBandwidth`|`6`|
/// |`Audio`|`8`|
/// |`Video`|`9`|
/// |`Data`|`18`|
/// |`Command`|`20`|
/// |`Other`|other numbers|
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    ChunkSize = 1,
    Acknowledgement = 3,
    UserControl,
    WindowAcknowledgementSize,
    PeerBandwidth,
    Audio = 8,
    Video = 9,
    Data = 18,
    Command = 20,
    Other = 0xff
}

impl From<u8> for MessageType {
    fn from(message_type: u8) -> Self {
        use MessageType::*;

        match message_type {
            1 => ChunkSize,
            3 => Acknowledgement,
            4 => UserControl,
            5 => WindowAcknowledgementSize,
            6 => PeerBandwidth,
            8 => Audio,
            9 => Video,
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
