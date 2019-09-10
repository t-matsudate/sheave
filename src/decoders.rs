use std::{
    collections::{
        HashMap
    },
    time::{
        Duration
    }
};
use crate::{
    messages::*,
};

pub(crate) trait RtmpDecoder: GetByteBuffer {
    fn decode_basic_header(&mut self) -> Option<BasicHeader>;
    fn decode_message_header(&mut self, message_format: MessageFormat) -> Option<MessageHeader>;
    fn decode_extended_timestamp(&mut self, message_header: MessageHeader) -> Option<Duration>;
    fn decode_chunk_size(&mut self) -> Option<ChunkData>;
    fn decode_bytes_read(&mut self) -> Option<ChunkData>;
    fn decode_ping(&mut self) -> Option<ChunkData>;
    fn decode_server_bandwidth(&mut self) -> Option<ChunkData>;
    fn decode_client_bandwidth(&mut self) -> Option<ChunkData>;
    fn decode_amf_number(&mut self) -> Option<AmfData>;
    fn decode_amf_boolean(&mut self) -> Option<AmfData>;
    fn decode_amf_string(&mut self) -> Option<AmfData>;
    fn decode_amf_object(&mut self) -> Option<AmfData>;
    fn decode_amf_null(&mut self) -> Option<AmfData>;
    fn decode_amf_data(&mut self) -> Option<AmfData>;
    fn decode_invoke_connect_result(&mut self) -> Option<ChunkData>;
    fn decode_invoke_release_stream_result(&mut self) -> Option<ChunkData>;
    fn decode_invoke_create_stream_result(&mut self) -> Option<ChunkData>;
    fn decode_invoke_on_fc_publish(&mut self) -> Option<ChunkData>;
    fn decode_invoke_on_status(&mut self) -> Option<ChunkData>;
    fn decode_invoke(&mut self) -> Option<ChunkData>;
    fn decode_unknown(&mut self) -> Option<ChunkData>;
    fn decode_chunk_data(&mut self, message_type: MessageType) -> Option<ChunkData>;
}

impl GetByteBuffer for ByteBuffer {
    fn get_u8(&mut self) -> Option<u8> {
        let offset = self.offset();
        let n = self.bytes().get(offset).map(|b| *b);

        self.offset_to(1);
        n
    }

    fn get_u16_be(&mut self) -> Option<u16> {
        let offset = self.offset();
        let n = self.bytes().get(offset..(offset + 2)).map(
            |bytes| {
                let mut ret: [u8; 2] = [0; 2];

                for i in 0..bytes.len() {
                    ret[i] = bytes[i];
                }

                u16::from_be_bytes(ret)
            }
        );

        self.offset_to(2);
        n
    }

    fn get_u16_le(&mut self) -> Option<u16> {
        let offset = self.offset();
        let n = self.bytes().get(offset..(offset + 2)).map(
            |bytes| {
                let mut ret: [u8; 2] = [0; 2];

                for i in 0..bytes.len() {
                    ret[i] = bytes[i];
                }

                u16::from_le_bytes(ret)
            }
        );

        self.offset_to(2);
        n
    }

    fn get_u24_be(&mut self) -> Option<u32> {
        let offset = self.offset();
        let n = self.bytes().get(offset..(offset + 3)).map(
            |bytes| {
                let mut ret: [u8; 4] = [0; 4];

                for i in 0..bytes.len() {
                    ret[i + 1] = bytes[i];
                }

                u32::from_be_bytes(ret)
            }
        );

        self.offset_to(3);
        n
    }

    fn get_u32_be(&mut self) -> Option<u32> {
        let offset = self.offset();
        let n = self.bytes().get(offset..(offset + 4)).map(
            |bytes| {
                let mut ret: [u8; 4] = [0; 4];

                for i in 0..bytes.len() {
                    ret[i] = bytes[i];
                }

                u32::from_be_bytes(ret)
            }
        );

        self.offset_to(4);
        n
    }

    fn get_u32_le(&mut self) -> Option<u32> {
        let offset = self.offset();
        let n = self.bytes().get(offset..(offset + 4)).map(
            |bytes| {
                let mut ret: [u8; 4] = [0; 4];

                for i in 0..bytes.len() {
                    ret[i] = bytes[i];
                }

                u32::from_le_bytes(ret)
            }
        );

        self.offset_to(4);
        n
    }

    fn get_f64(&mut self) -> Option<f64> {
        let offset = self.offset();
        let n = self.bytes().get(offset..(offset + 8)).map(
            |bytes| {
                let mut ret: [u8; 8] = [0; 8];

                for i in 0..bytes.len() {
                    ret[i] = bytes[i];
                }

                f64::from_bits(u64::from_be_bytes(ret))
            }
        );

        self.offset_to(8);
        n
    }

    fn get_sliced_bytes(&mut self, len: usize) -> Option<Vec<u8>> {
        let offset = self.offset();
        let bytes = self.bytes().get(offset..(offset + len)).map(|bytes| bytes.to_vec());

        self.offset_to(len);
        bytes
    }

    fn peek_byte(&self) -> Option<u8> {
        let offset = self.offset();

        self.bytes().get(offset).map(|byte| *byte)
    }

    fn peek_bytes(&self, len: usize) -> Option<Vec<u8>> {
        let offset = self.offset();

        self.bytes().get(offset..(offset + len)).map(|bytes| bytes.to_vec())
    }
}

impl RtmpDecoder for ByteBuffer {
    fn decode_basic_header(&mut self) -> Option<BasicHeader> {
        self.get_u8().map(
            |b| {
                let message_format: MessageFormat = ((b & BasicHeader::MESSAGE_HEADER_FORMAT) >> 6).into();
                let basic_header_type = b & BasicHeader::BASIC_HEADER_TYPE;

                match basic_header_type {
                    0 => BasicHeader::new(
                        message_format,
                        ChunkId::U8(self.get_u8().unwrap() + 64)
                    ),
                    1 => BasicHeader::new(
                        message_format,
                        ChunkId::U16(self.get_u16_le().unwrap() + 64)
                    ),
                    _ => BasicHeader::new(
                        message_format,
                        ChunkId::U8(basic_header_type)
                    )
                }
            }
        )
    }

    fn decode_message_header(&mut self, message_format: MessageFormat) -> Option<MessageHeader> {
        match message_format {
            MessageFormat::New => if self.offset() + MessageHeader::LEN_NEW - 1 >= self.len() {
                None
            } else {
                Some(MessageHeader::New {
                    timestamp: Duration::from_secs(self.get_u24_be().unwrap() as u64),
                    message_len: self.get_u24_be().unwrap(),
                    message_type: self.get_u8().unwrap().into(),
                    message_id: self.get_u32_le().unwrap()
                })
            },
            MessageFormat::SameSource => if self.offset() + MessageHeader::LEN_SAME_SOURCE - 1 >= self.len() {
                None
            } else {
                Some(MessageHeader::SameSource {
                    timestamp: Duration::from_secs(self.get_u24_be().unwrap() as u64),
                    message_len: self.get_u24_be().unwrap(),
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

    fn decode_extended_timestamp(&mut self, message_header: MessageHeader) -> Option<Duration> {
        message_header.get_timestamp().and_then(
            |timestamp| if timestamp.as_secs() > U24_MAX as u64 {
                self.get_u32_be().map(
                    |extended_timestamp| Duration::from_secs(extended_timestamp as u64)
                )
            } else {
                None
            }
        )
    }

    fn decode_chunk_size(&mut self) -> Option<ChunkData> {
        self.get_u32_be().map(
            |chunk_size| {
                if chunk_size & 0x80000000 != 0 {
                    warn!("The most significant bit is 1!");
                }

                ChunkData::ChunkSize(chunk_size)
            }
        )
    }

    fn decode_bytes_read(&mut self) -> Option<ChunkData> {
        self.get_u32_be().map(|bytes_read| ChunkData::BytesRead(bytes_read))
    }

    fn decode_ping(&mut self) -> Option<ChunkData> {
        self.get_u16_be().and_then(
            |ping_type_id| {
                use crate::messages::PingType::*;

                let ping_type: PingType = (ping_type_id as u8).into();

                match ping_type {
                    StreamBegin => self.get_u32_be().map(|stream_id| ChunkData::Ping(PingData::StreamBegin(stream_id))),
                }
            }
        )
    }

    fn decode_server_bandwidth(&mut self) -> Option<ChunkData> {
        self.get_u32_be().map(
            |bandwidth| ChunkData::ServerBandwidth(bandwidth)
        )
    }

    fn decode_client_bandwidth(&mut self) -> Option<ChunkData> {
        self.get_u32_be().and_then(
            |bandwidth| self.get_u8().map(
                |limit_type| ChunkData::ClientBandwidth(bandwidth, limit_type.into())
            )
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
        }

        Some(AmfData::Object(object))
    }

    fn decode_amf_null(&mut self) -> Option<AmfData> {
        self.get_u8().map(|_| AmfData::Null) // AMF0's Null has only its type id.
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
                    Null => self.decode_amf_null(),
                    _ => None
                }
            }
        )
    }

    fn decode_invoke_connect_result(&mut self) -> Option<ChunkData> {
        self.decode_amf_data().and_then(
            |s| s.string().and_then(
                |command| self.decode_amf_data().and_then(
                    |n| n.number().map(
                        |transaction_id| {
                            let result: NetConnectionResult = command.into();
                            let properties = self.decode_amf_data().and_then(
                                |data| data.object().map(
                                    |properties| properties
                                )
                            ).unwrap_or(HashMap::new());
                            let information: InfoObject = self.decode_amf_data().and_then(
                                |data| data.object().map(
                                    |information| information.into()
                                )
                            ).unwrap_or(InfoObject::new());

                            ChunkData::Invoke(
                                InvokeCommand::NetConnection(
                                    NetConnectionCommand::ConnectResult {
                                        result,
                                        transaction_id: transaction_id as u64,
                                        properties,
                                        information
                                    }
                                )
                            )
                        }
                    )
                )
            )
        )
    }

    fn decode_invoke_release_stream_result(&mut self) -> Option<ChunkData> {
        self.decode_amf_data().and_then(
            |s| s.string().and_then(
                |command| self.decode_amf_data().and_then(
                    |n| n.number().map(
                        |transaction_id| {
                            self.decode_amf_null(); // AMF's Null. (this has only AMF's type id)

                            let result: NetConnectionResult = command.into();

                            ChunkData::Invoke(
                                InvokeCommand::NetConnection(
                                    NetConnectionCommand::ReleaseStreamResult {
                                        result,
                                        transaction_id: transaction_id as u64
                                    }
                                )
                            )
                        }
                    )
                )
            )
        )
    }

    fn decode_invoke_create_stream_result(&mut self) -> Option<ChunkData> {
        self.decode_amf_data().and_then(
            |s| s.string().and_then(
                |command| self.decode_amf_data().and_then(
                    |n| n.number().map(
                        |transaction_id| {
                            self.decode_amf_null(); // AMF's Null. (this has only AMF's type id)

                            let result: NetConnectionResult = command.into();
                            let message_id = self.decode_amf_data().and_then(
                                |n| n.number()
                            ).unwrap();

                            ChunkData::Invoke(
                                InvokeCommand::NetConnection(
                                    NetConnectionCommand::CreateStreamResult {
                                        result,
                                        transaction_id: transaction_id as u64,
                                        message_id: message_id as u64 as u32
                                    }
                                )
                            )
                        }
                    )
                )
            )
        )
    }

    fn decode_invoke_on_fc_publish(&mut self) -> Option<ChunkData> {
        self.decode_amf_data().and_then(
            |s| s.string().and_then(
                |command| if command == "onFCPublish" {
                    Some(
                        ChunkData::Invoke(
                            InvokeCommand::FcPublish(
                                FcPublishCommand::OnFcPublish
                            )
                        )
                    )
                } else {
                    None
                }
            )
        )
    }

    fn decode_invoke_on_status(&mut self) -> Option<ChunkData> {
        self.decode_amf_data().and_then(
            |s| s.string().and_then(
                |command| self.decode_amf_data().and_then(
                    |n| n.number().and_then(
                        |transaction_id| if command != "onStatus" {
                            None
                        } else {
                            self.decode_amf_null(); // AMF's Null. (this has only AMF's type id)

                            let info_object: InfoObject = self.decode_amf_data().and_then(
                                |data| data.object().map(
                                    |info_object| info_object.into()
                                )
                            ).unwrap();

                            Some(
                                ChunkData::Invoke(
                                    InvokeCommand::NetStream(
                                        NetStreamCommand::OnStatus {
                                            transaction_id: transaction_id as u64,
                                            info_object: info_object
                                        }
                                    )
                                )
                            )
                        }
                    )
                )
            )
        )
    }

    fn decode_invoke(&mut self) -> Option<ChunkData> {
        self.decode_amf_data().and_then(
            |s| s.string().and_then(
                |command| self.decode_amf_data().and_then(
                    |n| n.number().and_then(
                        |transaction_id| {
                            use crate::messages::{
                                ChunkData::Invoke,
                                InvokeCommand::*,
                                NetConnectionCommand::*,
                                NetStreamCommand::*,
                                FcPublishCommand as fc
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

                                Some(
                                    Invoke(
                                        NetConnection(
                                            Connect {
                                                argument: None,
                                                transaction_id: transaction_id as u64,
                                                command_object
                                            }
                                        )
                                    )
                                )
                            } else if command == "releaseStream" {
                                self.decode_amf_null(); // AMF's Null. (this has only AMF's type id)

                                let play_path = self.decode_amf_data().and_then(
                                    |data| data.string()
                                ).unwrap();

                                Some(
                                    Invoke(
                                        NetConnection(
                                            ReleaseStream {
                                                transaction_id: transaction_id as u64,
                                                play_path
                                            }
                                        )
                                    )
                                )
                            } else if command == "createStream" {
                                self.decode_amf_null(); // AMF's Null. (this has only AMF's type id)

                                Some(
                                    Invoke(
                                        NetConnection(
                                            CreateStream {
                                                transaction_id: transaction_id as u64
                                            }
                                        )
                                    )
                                )
                            } else if command == "FCPublish" {
                                self.decode_amf_null(); // AMF's Null. (this has only AMF's type id)

                                let play_path = self.decode_amf_data().and_then(
                                    |data| data.string()
                                ).unwrap();

                                Some(
                                    Invoke(
                                        FcPublish(
                                            fc::FcPublish {
                                                transaction_id: transaction_id as u64,
                                                play_path
                                            }
                                        )
                                    )
                                )
                            } else if command == "publish" {
                                self.decode_amf_null(); // AMF's Null. (this has only AMF's type id)

                                let play_path = self.decode_amf_data().and_then(
                                    |data| data.string()
                                ).unwrap();
                                let play_type: PlayType = self.decode_amf_data().and_then(
                                    |data| data.string().map(
                                        |play_type| play_type.into()
                                    )
                                ).unwrap();

                                Some(
                                    Invoke(
                                        NetStream(
                                            Publish {
                                                transaction_id: transaction_id as u64,
                                                play_path,
                                                play_type
                                            }
                                        )
                                    )
                                )
                            } else {
                                info!("Unknown invoke command: {}", command);

                                let len = self.len();
                                let offset = self.offset();

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

    fn decode_chunk_data(&mut self, message_type: MessageType) -> Option<ChunkData> {
        use crate::messages::MessageType::*;

        match message_type {
            ChunkSize => self.decode_chunk_size(),
            BytesRead => self.decode_bytes_read(),
            Ping => self.decode_ping(),
            ServerBandwidth => self.decode_server_bandwidth(),
            ClientBandwidth => self.decode_client_bandwidth(),
            Invoke => self.decode_invoke(),
            _ => self.decode_unknown()
        }
    }
}
