use std::{
    collections::{
        HashMap
    },
    io::{
        Error as IOError,
        ErrorKind,
        Read,
        Result as IOResult,
    },
    net::{
        TcpStream
    },
    time::{
        Duration
    }
};
use crate::{
    errors::{
        ChunkLengthError
    },
    messages::*
};

pub(self) trait RtmpDecoder: GetByteBuffer {
    fn decode_basic_header(&mut self) -> Option<BasicHeader>;
    fn decode_message_header(&mut self, basic_header: BasicHeader) -> Option<MessageHeader>;
    fn decode_extended_timestamp(&mut self, message_header: MessageHeader) -> Option<Duration>;
    fn decode_server_bandwidth(&mut self) -> Option<ChunkData>;
    fn decode_amf_number(&mut self) -> Option<AmfData>;
    fn decode_amf_boolean(&mut self) -> Option<AmfData>;
    fn decode_amf_string(&mut self) -> Option<AmfData>;
    fn decode_amf_object(&mut self) -> Option<AmfData>;
    fn decode_amf_data(&mut self) -> Option<AmfData>;
    fn decode_invoke(&mut self) -> Option<ChunkData>;
    fn decode_unknown(&mut self) -> Option<ChunkData>;
    fn decode_chunk_data(&mut self, message_header: &MessageHeader) -> Option<ChunkData>;
} 

impl GetByteBuffer for ByteBuffer {
    fn get_u8(&mut self) -> Option<u8> {
        let offset = self.offset();

        if offset == self.len() {
            return None;
        }

        let byte = self.bytes()[offset];

        self.offset_to(1);
        Some(byte)
    }

    fn get_u16_be(&mut self) -> Option<u16> {
        let offset = self.offset();

        if offset + 1 >= self.len() {
            return None;
        }

        let mut bytes: [u8; 2] = [0; 2];
        let s = &self.bytes()[offset..(offset + 2)];

        for i in 0..bytes.len() {
            bytes[i] = s[i];
        }

        self.offset_to(2);
        Some(u16::from_be_bytes(bytes))
    }

    fn get_u16_le(&mut self) -> Option<u16> {
        let offset = self.offset();

        if offset + 1 >= self.len() {
            return None;
        }

        let mut bytes: [u8; 2] = [0; 2];
        let s = &self.bytes()[offset..(offset + 2)];

        for i in 0..bytes.len() {
            bytes[i] = s[i];
        }

        self.offset_to(2);
        Some(u16::from_le_bytes(bytes))
    }

    fn get_u24_be(&mut self) -> Option<u32> {
        let offset = self.offset();

        if offset + 2 >= self.len() {
            return None;
        }

        let mut bytes: [u8; 4] = [0; 4];
        let s = &self.bytes()[offset..(offset + 3)];

        for i in 1..bytes.len() {
            bytes[i] = s[i - 1];
        }

        self.offset_to(3);
        Some(u32::from_be_bytes(bytes))
    }

    fn get_u32_be(&mut self) -> Option<u32> {
        let offset = self.offset();

        if offset + 3 >= self.len() {
            return None;
        }

        let mut bytes: [u8; 4] = [0; 4];
        let s = &self.bytes()[offset..(offset + 4)];

        for i in 0..bytes.len() {
            bytes[i] = s[i];
        }

        self.offset_to(4);
        Some(u32::from_be_bytes(bytes))
    }

    fn get_u32_le(&mut self) -> Option<u32> {
        let offset = self.offset();

        if offset + 3 >= self.len() {
            return None;
        }

        let mut bytes: [u8; 4] = [0; 4];
        let s = &self.bytes()[offset..(offset + 4)];

        for i in 0..bytes.len() {
            bytes[i] = s[i];
        }

        self.offset_to(4);
        Some(u32::from_le_bytes(bytes))
    }

    fn get_f64(&mut self) -> Option<f64> {
        let offset = self.offset();

        if offset + 7 >= self.len() {
            return None;
        }

        let mut bytes: [u8; 8] = [0; 8];
        let s = &self.bytes()[offset..(offset + 8)];

        for i in 0..bytes.len() {
            bytes[i] = s[i];
        }

        self.offset_to(8);
        Some(f64::from_bits(u64::from_be_bytes(bytes)))
    }

    fn get_sliced_bytes(&mut self, len: usize) -> Option<Vec<u8>> {
        let offset = self.offset();

        if offset + len - 1 >= self.len() {
            return None;
        }

        let bytes = self.bytes()[offset..(offset + len)].to_vec();

        self.offset_to(len);
        Some(bytes)
    }

    fn peek_byte(&mut self) -> Option<u8> {
        let offset = self.offset();

        if offset == self.len() {
            return None;
        }

        Some(self.bytes()[offset])
    }

    fn peek_bytes(&mut self, len: usize) -> Option<Vec<u8>> {
        let offset = self.offset();

        if offset + len - 1 >= self.len() {
            return None;
        }

        Some(self.bytes()[offset..(offset + len)].to_vec())
    }
}

impl RtmpDecoder for ByteBuffer {
    fn decode_basic_header(&mut self) -> Option<BasicHeader> {
        self.get_u8().map(
            |b| {
                use crate::messages::BasicHeader::*;

                let message_format: MessageFormat = ((b & BasicHeader::MESSAGE_HEADER_FORMAT) >> 6).into();
                let basic_header_type = b & BasicHeader::BASIC_HEADER_TYPE;

                match basic_header_type {
                    0 => TwoBytes {
                        message_format,
                        channel_id: self.get_u8().unwrap() + 64
                    },
                    1 => ThreeBytes {
                        message_format,
                        channel_id: self.get_u16_le().unwrap() + 64
                    },
                    _ => OneByte {
                        message_format,
                        channel_id: basic_header_type
                    }
                }
            }
        )
    }

    fn decode_message_header(&mut self, basic_header: &BasicHeader) -> Option<MessageHeader> {
        match basic_header.get_message_format() {
            MessageFormat::New => if self.offset() + MessageHeader::LEN_NEW - 1 >= self.len() {
                None
            } else {
                Some(MessageHeader::New {
                    timestamp: Duration::from_secs(self.get_u24_be().unwrap() as u64),
                    message_len: self.get_u24_be().unwrap() as usize,
                    message_type: self.get_u8().unwrap().into(),
                    channel_id: self.get_u32_le().unwrap()
                })
            },
            MessageFormat::SameSource => if self.offset() + MessageHeader::LEN_SAME_SOURCE - 1 >= self.len() {
                None
            } else {
                Some(MessageHeader::SameSource {
                    timestamp: Duration::from_secs(self.get_u24_be().unwrap() as u64),
                    message_len: self.get_u24_be().unwrap() as usize,
                    message_type: self.get_u8().unwrap().into()
                })
            },
            MessageFormat::TimerChange => if self.offset() + MessageHeader::LEN_TIMER_CHANGE - 1 >= self.len() {
                None
            } else {
                Some(MessageHeader::TimerChange {
                    timestamp: Duration::from_secs(self.get_u24_be().unwrap() as u64)
                })
            },
            MessageFormat::Continue => Some(MessageHeader::Continue)
        }
    }

    fn decode_extended_timestamp(&mut self, message_header: &MessageHeader) -> Option<Duration> {
        message_header.get_timestamp().and_then(
            |timestamp| if timestamp.as_secs() >= (U24_MAX as u64) {
                Some(Duration::from_secs(self.get_u32_be().unwrap() as u64))
            } else {
                None
            }
        )
    }

    fn decode_server_bandwidth(&mut self) -> Option<ChunkData> {
        self.get_u32_be().map(
            |size| ChnukData::ServerBandwidth(size as usize)
        )
    }

    fn decode_amf_number(&mut self) -> Option<AmfData> {
        self.get_f64().map(
            |number| AmfData::Number(number)
        )
    }

    fn decode_amf_boolean(&mut self) -> Option<AmfData> {
        self.get_u8().map(
            |b| AmfData::Boolean(b == 1)
        )
    }

    fn decode_amf_string(&mut self) -> Option<AmfData> {
        self.get_u16_be().and_then(
            |len| if len < 1 {
                Some(AmfData::String(String::new()))
            } else if let Ok(s) = String::from_utf8(self.get_sliced_bytes(len as usize).unwrap()) {
                Some(AmfData::String(s))
            } else {
                None
            }
        )
    }

    fn decode_amf_object(&mut self) -> Option<AmfData> {
        let mut object: HashMap<String, AmfData> = HashMap::new();

        while let Some(bytes) = self.peek_bytes(3) {
            if bytes == &AmfData::OBJECT_END_SEQUENCE {
                break;
            }

            let key = self.decode_amf_string().unwrap().string().unwrap();
            let value = self.decode_amf_data().unwrap();

            object.insert(key, value);

            let byte = self.peek_byte().unwrap();

            // a mysterious 'ﾃ'. Somehow this will appear before the object end sequence(== [0, 0, 9]).
            // Currently this should skip because can't decode.
            if byte == (0xc3 as u8) {
                self.offset_to(1);
            }
        }

        Some(AmfData::Object(object))
    }

    fn decode_amf_data(&mut self) -> Option<AmfData> {
        self.get_u8().and_then(
            |b| {
                use crate::messages::AmfDataType::*;
                use crate::messages::AmfDataType::String as AmfString;

                let amf_type: AmfDataType = b.into();

                match amf_type {
                    Number => self.decode_amf_number(),
                    Boolean => self.decode_amf_boolean(),
                    AmfString => self.decode_amf_string(),
                    Object => self.decode_amf_object(),
                    _ => None
                }
            }
        )
    }

    fn decode_invoke(&mut self) -> Option<ChunkData> {
        let offset = self.offset();
        println!("{:x?}", &self.bytes()[offset..]);
        self.decode_amf_data().and_then(
            |s| s.string().and_then(
                |command| self.decode_amf_data().and_then(
                    |n| n.number().and_then(
                        |transaction_id| {
                            use crate::messages::{
                                ChunkData::Invoke,
                                InvokeCommand::*,
                                NetConnectionCommand::*
                            };

                            if command == "connect" {
                                let command_object = self.decode_amf_data().and_then(
                                    |data| data.object().map(
                                        |object| {
                                            let command_object: CommandObject = object.into();

                                            command_object
                                        }
                                    )
                                ).unwrap();

                                Some(Invoke(NetConnection(Connect {
                                    argument: None,
                                    transaction_id: transaction_id as u64,
                                    command_object
                                })))
                            } else {
                                info!("Unknown invoke command: {}", command);

                                let len = self.len();

                                self.get_sliced_bytes(len - offset).map(
                                    |bytes| Invoke(Unknown(bytes))
                                )
                            }
                        }
                    )
                )
            )
        )
    }

    fn decode_unknown(&mut self) -> Option<ChunkData> {
        use crate::messages::ChunkData::Unknown;

        let len = self.len();
        let offset = self.offset();
        Some(Unknown(self.get_sliced_bytes(len - offset).unwrap()))
    }

    fn decode_chunk_data(&mut self, message_header: &MessageHeader) -> Option<ChunkData> {
        message_header.get_message_type().and_then(
            |message_type| {
                use crate::messages::MessageType::*;

                match message_type {
                    Invoke => self.decode_invoke(),
                    _ => self.decode_unknown()
                }
            }
        )
    }
}

pub(crate) fn decode_chunk(stream: &mut TcpStream) -> IOResult<Chunk> {
    let mut v: Vec<u8> = Vec::new();
    let mut bytes: [u8; 1024] = [0; 1024];
    let mut remain = usize::default();

    while let Ok(read) = stream.read(&mut bytes) {
        remain = read;

        if read != 1024 {
            break;
        }

        v.extend_from_slice(&bytes);
    }

    v.extend_from_slice(&bytes[..remain]);

    let mut buffer = ByteBuffer::new(v);

    buffer.decode_basic_header().and_then(
        |basic_header| buffer.decode_message_header(&basic_header).map(
            |message_header| {
                let extended_timestamp = buffer.decode_extended_timestamp(&message_header);
                let chunk_data = buffer.decode_chunk_data(&message_header);

                Chunk::new(basic_header, message_header, extended_timestamp, chunk_data)
                }
            )
    ).ok_or(
        IOError::new(ErrorKind::InvalidInput, ChunkLengthError::new("The chunk is incomplete.".to_string(), None))
    )
}
