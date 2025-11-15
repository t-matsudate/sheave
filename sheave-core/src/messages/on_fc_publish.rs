use std::io::Result as IOResult;
use super::{
    Channel,
    ChunkData,
    Command,
    headers::MessageType
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer,
};

/// The response message for FcPublish requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnFcPublish;

impl ChunkData for OnFcPublish {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for OnFcPublish {}

impl Decoder<OnFcPublish> for ByteBuffer {
    /// Decodes bytes into an OnFcPublish command.
    fn decode(&mut self) -> IOResult<OnFcPublish> {
        Ok(OnFcPublish)
    }
}

impl Encoder<OnFcPublish> for ByteBuffer {
    /// Encodes an OnFcPublish command into bytes.
    /// However this encodes nothing because has no field.
    fn encode(&mut self, _: &OnFcPublish) {
        return
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_on_fc_publish() {
        let mut buffer = ByteBuffer::default();
        let result: IOResult<OnFcPublish> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(OnFcPublish, actual)
    }

    #[test]
    fn encode_on_fc_publish() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&OnFcPublish);
    }
}
