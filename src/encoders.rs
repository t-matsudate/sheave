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

pub(crate) trait RtmpEncoder: PutByteBuffer {
    fn encode_basic_header(&mut self, basic_header: BasicHeader);
    fn encode_message_header(&mut self, message_header: MessageHeader);
    fn encode_extended_timestamp(&mut self, timestamp: Option<Duration>);
    fn encode_chunk_size(&mut self, chunk_size: u32);
    fn encode_bytes_read(&mut self, bytes_read: u32);
    fn encode_ping(&mut self, ping_data: PingData);
    fn encode_server_bandwidth(&mut self, bandwidth: u32);
    fn encode_client_bandwidth(&mut self, bandwidth: u32, limit_type: LimitType);
    fn encode_amf_number(&mut self, number: f64);
    fn encode_amf_boolean(&mut self, boolean: bool);
    fn encode_amf_string(&mut self, string: String);
    fn encode_amf_object(&mut self, object: HashMap<String, AmfData>);
    fn encode_amf_null(&mut self);
    fn encode_amf_data(&mut self, data: AmfData);
    fn encode_invoke_net_connection(&mut self, net_connection: NetConnectionCommand);
    fn encode_invoke_fc_publish(&mut self, fc_publish: FcPublishCommand);
    fn encode_invoke(&mut self, invoke: InvokeCommand);
    fn encode_unknown(&mut self, unknown: Vec<u8>);
    fn encode_chunk_data(&mut self, chunk_data: Option<ChunkData>);
}

impl PutByteBuffer for ByteBuffer {
    fn put_u8(&mut self, byte: u8) {
        self.bytes_mut().push(byte);
        self.add_len(1);
    }

    fn put_u16_be(&mut self, byte: u16) {
        let bytes = byte.to_be_bytes();

        self.bytes_mut().extend_from_slice(&bytes);
        self.add_len(2);
    }

    fn put_u16_le(&mut self, byte: u16) {
        let bytes = byte.to_le_bytes();

        self.bytes_mut().extend_from_slice(&bytes);
        self.add_len(2);
    }

    fn put_u24_be(&mut self, byte: u32) {
        let bytes = byte.to_be_bytes();

        self.bytes_mut().extend_from_slice(&bytes[1..]);
        self.add_len(3);
    }

    fn put_u32_be(&mut self, byte: u32) {
        let bytes = byte.to_be_bytes();

        self.bytes_mut().extend_from_slice(&bytes);
        self.add_len(4);
    }

    fn put_u32_le(&mut self, byte: u32) {
        let bytes = byte.to_le_bytes();

        self.bytes_mut().extend_from_slice(&bytes);
        self.add_len(4);
    }

    fn put_f64(&mut self, byte: f64) {
        let bytes = byte.to_bits().to_be_bytes();

        self.bytes_mut().extend_from_slice(&bytes);
        self.add_len(8);
    }

    fn put_bytes(&mut self, mut bytes: Vec<u8>) {
        let len = bytes.len();

        self.bytes_mut().append(&mut bytes);
        self.add_len(len);
    }
}

impl RtmpEncoder for ByteBuffer {
    fn encode_basic_header(&mut self, basic_header: BasicHeader) {
        use crate::messages::ChunkId::*;

        match basic_header.get_chunk_id() {
            U8(chunk_id) => {
                if chunk_id < 64 {
                    self.put_u8(((basic_header.get_message_format() as u8) << 6) | chunk_id);
                } else {
                    self.put_u8((basic_header.get_message_format() as u8) << 6);
                    self.put_u8(chunk_id - 64);
                };
            },
            U16(chunk_id) => {
                self.put_u8((basic_header.get_message_format() as u8) + 1);
                self.put_u16_le(chunk_id - 64);
            }
        }
    }

    fn encode_message_header(&mut self, message_header: MessageHeader) {
        message_header.get_timestamp().map(
            |timestamp| self.put_u24_be(timestamp.as_secs() as u32)
        );
        message_header.get_message_len().map(
            |message_len| self.put_u24_be(message_len)
        );
        message_header.get_message_type().map(
            |message_type| self.put_u8(message_type as u8)
        );
        message_header.get_message_id().map(
            |message_id| self.put_u32_le(message_id)
        );
    }

    fn encode_extended_timestamp(&mut self, timestamp: Option<Duration>) {
        timestamp.map(
            |timestamp| self.put_u32_be(timestamp.as_secs() as u32)
        );
    }

    fn encode_chunk_size(&mut self, chunk_size: u32) {
        self.put_u32_be(chunk_size & 0x7fffffff);
    }

    fn encode_bytes_read(&mut self, bytes_read: u32) {
        self.put_u32_be(bytes_read);
    }

    fn encode_ping(&mut self, ping_data: PingData) {
        match ping_data {
            PingData::StreamBegin(stream_id) => {
                self.put_u16_be(PingType::StreamBegin as u16);
                self.put_u32_be(stream_id);
            },
        }
    }

    fn encode_server_bandwidth(&mut self, bandwidth: u32) {
        self.put_u32_be(bandwidth);
    }

    fn encode_client_bandwidth(&mut self, bandwidth: u32, limit_type: LimitType) {
        self.put_u32_be(bandwidth);
        self.put_u8(limit_type as u8);
    }

    fn encode_amf_number(&mut self, number: f64) {
        self.put_u8(AmfDataType::Number as u8);
        self.put_f64(number);
    }

    fn encode_amf_boolean(&mut self, boolean: bool) {
        self.put_u8(AmfDataType::Boolean as u8);
        self.put_u8(boolean as u8);
    }

    fn encode_amf_string(&mut self, string: String) {
        self.put_u8(AmfDataType::String as u8);
        self.put_u16_be(string.len() as u16);
        self.put_bytes(string.into_bytes());
    }

    fn encode_amf_object(&mut self, object: HashMap<String, AmfData>) {
        self.put_u8(AmfDataType::Object as u8);

        for (key, value) in object {
            self.put_u16_be(key.len() as u16);
            self.put_bytes(key.into_bytes());
            self.encode_amf_data(value);
        }

        self.put_bytes(AmfData::OBJECT_END_SEQUENCE.to_vec());
    }

    fn encode_amf_null(&mut self) {
        self.put_u8(AmfDataType::Null as u8);
    }

    fn encode_amf_data(&mut self, data: AmfData) {
        use crate::messages::AmfData::*;

        match data {
            Number(number) => self.encode_amf_number(number),
            Boolean(boolean) => self.encode_amf_boolean(boolean),
            String(string) => self.encode_amf_string(string),
            Null => self.encode_amf_null(),
            Object(object) => self.encode_amf_object(object),
            _ => ()
        }
    }

    fn encode_invoke_net_connection(&mut self, net_connection: NetConnectionCommand) {
        use crate::messages::NetConnectionCommand::*;

        match net_connection {
            Connect {
                transaction_id,
                command_object,
                argument: _
            } => {
                self.encode_amf_string("connect".to_string());
                self.encode_amf_number(transaction_id as f64);
                self.encode_amf_object(command_object.into());
            },
            ConnectResult {
                result,
                transaction_id,
                properties,
                information
            } => {
                self.encode_amf_string(result.into());
                self.encode_amf_number(transaction_id as f64);
                self.encode_amf_object(properties);
                self.encode_amf_object(information);
            },
            ReleaseStream {
                transaction_id,
                play_path
            } => {
                self.encode_amf_string("releaseStream".to_string());
                self.encode_amf_number(transaction_id as f64);
                self.encode_amf_null();
                self.encode_amf_string(play_path);
            },
            ReleaseStreamResult {
                result,
                transaction_id
            } => {
                self.encode_amf_string(result.into());
                self.encode_amf_number(transaction_id as f64);
                self.encode_amf_null();
            },
            CreateStream {
                transaction_id
            } => {
                self.encode_amf_string("createStream".to_string());
                self.encode_amf_number(transaction_id as f64);
                self.encode_amf_null();
            },
            CreateStreamResult {
                result,
                message_id,
                transaction_id
            } => {
                self.encode_amf_string(result.into());
                self.encode_amf_number(transaction_id as f64);
                self.encode_amf_null();
                self.encode_amf_number(message_id as u64 as f64);
            }
        }
    }

    fn encode_invoke_fc_publish(&mut self, fc_publish: FcPublishCommand) {
        use crate::messages::FcPublishCommand::*;

        match fc_publish {
            FcPublish {
                transaction_id,
                play_path
            } => {
                self.encode_amf_string("FCPublish".to_string());
                self.encode_amf_number(transaction_id as f64);
                self.encode_amf_null();
                self.encode_amf_string(play_path);
            },
            OnFcPublish => {
                self.encode_amf_string("onFCPublish".to_string());
            }
        }
    }

    fn encode_invoke(&mut self, invoke: InvokeCommand) {
        use crate::messages::InvokeCommand::*;

        match invoke {
            NetConnection(net_connection) => self.encode_invoke_net_connection(net_connection),
            FcPublish(fc_publish) => self.encode_invoke_fc_publish(fc_publish),
            Unknown(bytes) => self.put_bytes(bytes)
        }
    }

    fn encode_unknown(&mut self, unknown: Vec<u8>) {
        self.put_bytes(unknown);
    }

    fn encode_chunk_data(&mut self, chunk_data: Option<ChunkData>) {
        chunk_data.map(
            |chunk_data| {
                use crate::messages::ChunkData::*;

                match chunk_data {
                    ChunkSize(chunk_size) => self.encode_chunk_size(chunk_size),
                    BytesRead(bytes_read) => self.encode_bytes_read(bytes_read),
                    Ping(ping_data) => self.encode_ping(ping_data),
                    ServerBandwidth(bandwidth) => self.encode_server_bandwidth(bandwidth),
                    ClientBandwidth(bandwidth, limit_type) => self.encode_client_bandwidth(bandwidth, limit_type),
                    Invoke(invoke_command) => self.encode_invoke(invoke_command),
                    Unknown(bytes) => self.encode_unknown(bytes)
                }
            }
        );
    }
}

pub(crate) fn encode_chunk(chunk: Chunk) -> Vec<u8> {
    let mut buffer = ByteBuffer::new(Vec::new());

    buffer.encode_basic_header(chunk.get_basic_header());
    buffer.encode_message_header(chunk.get_message_header());
    buffer.encode_extended_timestamp(chunk.get_extended_timestamp());
    buffer.encode_chunk_data(chunk.get_chunk_data().clone());
    buffer.bytes().to_vec()
}
