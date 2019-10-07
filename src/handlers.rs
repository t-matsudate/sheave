use std::{
    cmp::{
        min
    },
    collections::{
        HashMap
    },
    io::{
        Error as IOError,
        ErrorKind,
        Read,
        Result as IOResult,
        Write
    },
    net::{
        TcpStream
    },
    time::{
        Duration,
        SystemTime
    }
};
use crate::{
    messages::*,
    decoders::*,
    encoders::*,
    handshake::*,
    errors::*,
    flv::*
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum RtmpState {
    TcpConnect,
    ReceivedHandshake,
    HandshakeDone,
    ReceivedConnect,
    SentConnectResult,
    ReceivedReleaseStream,
    SentReleaseStreamResult,
    ReceivedFcPublish,
    SentOnFcPublish,
    ReceivedCreateStream,
    SentCreateStreamResult,
    ReceivedPublish,
    Connected,
    Disconnecting,
    Disconnected,
    Error = 0xff
}

impl From<u8> for RtmpState {
    fn from(state: u8) -> Self {
        use RtmpState::*;

        match state {
            0x00 => TcpConnect,
            0x01 => ReceivedHandshake,
            0x02 => HandshakeDone,
            0x03 => ReceivedConnect,
            0x04 => SentConnectResult,
            0x05 => ReceivedReleaseStream,
            0x06 => SentReleaseStreamResult,
            0x07 => ReceivedFcPublish,
            0x08 => SentOnFcPublish,
            0x09 => ReceivedCreateStream,
            0x0a => SentCreateStreamResult,
            0x0b => ReceivedPublish,
            0x0c => Connected,
            0x0d => Disconnecting,
            0x0e => Disconnected,
            0xff => Error,
            _ => panic!("Undefined RTMP state!")
        }
    }
}

impl Default for RtmpState {
    fn default() -> Self {
        RtmpState::TcpConnect
    }
}

#[derive(Debug)]
pub(crate) struct LastChunk {
    message_type: Option<MessageType>,
    message_id: Option<u32>,
    message_len: Option<u32>,
    timestamp: Option<Duration>,
    chunk_data: Option<ChunkData>
}

impl LastChunk {
    fn new(message_type: Option<MessageType>, message_id: Option<u32>, message_len: Option<u32>, timestamp: Option<Duration>, chunk_data: Option<ChunkData>) -> Self {
        LastChunk {
            message_type,
            message_id,
            message_len,
            timestamp,
            chunk_data
        }
    }

    fn get_message_type(&self) -> Option<MessageType> {
        self.message_type
    }

    fn set_message_type(&mut self, message_type: Option<MessageType>) {
        self.message_type = message_type;
    }

    fn get_message_id(&self) -> Option<u32> {
        self.message_id
    }

    fn set_message_id(&mut self, message_id: Option<u32>) {
        self.message_id = message_id;
    }

    fn get_message_len(&self) -> Option<u32> {
        self.message_len
    }

    fn set_message_len(&mut self, message_len: Option<u32>) {
        self.message_len = message_len;
    }

    fn get_timestamp(&self) -> Option<Duration> {
        self.timestamp
    }

    fn set_timestamp(&mut self, timestamp: Option<Duration>) {
        self.timestamp = timestamp;
    }

    fn get_chunk_data(&self) -> &Option<ChunkData> {
        &self.chunk_data
    }

    fn set_chunk_data(&mut self, chunk_data: Option<ChunkData>) {
        self.chunk_data = chunk_data;
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(self) enum Channel {
    Network = 0x02,
    System,
    Audio,
    Video = 0x06
}

impl From<u8> for Channel {
    fn from(channel_id: u8) -> Self {
        use Channel::*;

        match channel_id {
            0x02 => Network,
            0x03 => System,
            0x04 => Audio,
            0x06 => Video,
            _ => panic!("Undefined channel id!")
        }
    }
}

#[derive(Debug)]
pub(crate) struct RtmpHandler {
    limit_type: LimitType,
    state: RtmpState,
    last_received_chunk_id: ChunkId,
    last_sent_chunk_id: ChunkId,
    message_id: u32,
    chunk_size: u32,
    bytes_read: u32,
    server_bandwidth: u32,
    client_bandwidth: u32,
    transaction_id: u64,
    play_path: String,
    play_type: PlayType,
    stream: TcpStream,
    handshake: RtmpHandshake,
    command_object: CommandObject,
    info_object: InfoObject,
    flv: Flv,
    received_chunks: HashMap<ChunkId, LastChunk>,
    sent_chunks: HashMap<ChunkId, LastChunk>
}

impl RtmpHandler {
    const DEFAULT_CHUNK_SIZE: u32 = 128;
    const DEFAULT_BANDWIDTH: u32 = 3000000;

    pub(crate) fn new(start_time: Duration, stream: TcpStream) -> Self {
        let limit_type = LimitType::default();
        let state = RtmpState::default();
        let last_received_chunk_id = ChunkId::default();
        let last_sent_chunk_id = ChunkId::default();
        let message_id = u32::default();
        let chunk_size = Self::DEFAULT_CHUNK_SIZE;
        let bytes_read = u32::default();
        let server_bandwidth = Self::DEFAULT_BANDWIDTH;
        let client_bandwidth = Self::DEFAULT_BANDWIDTH;
        let transaction_id = u64::default();
        let play_path = String::default();
        let play_type = PlayType::default();
        let handshake = RtmpHandshake::new(start_time);
        let command_object = CommandObject::default();
        let info_object = InfoObject::default();
        let flv = Flv::default();
        let received_chunks = HashMap::default();
        let sent_chunks = HashMap::default();

        RtmpHandler {
            limit_type,
            state,
            last_received_chunk_id,
            last_sent_chunk_id,
            message_id,
            chunk_size,
            bytes_read,
            server_bandwidth,
            client_bandwidth,
            transaction_id,
            play_path,
            play_type,
            stream,
            handshake,
            command_object,
            info_object,
            flv,
            received_chunks,
            sent_chunks
        }
    }

    pub(crate) fn get_state(&self) -> RtmpState {
        self.state
    }

    fn receive_c0c1(&mut self) -> IOResult<()> {
        let mut tmp: [u8; VERSION_CHUNK_SIZE + HANDSHAKE_CHUNK_SIZE] = [0; VERSION_CHUNK_SIZE + HANDSHAKE_CHUNK_SIZE];
        let mut bytes: Vec<u8> = Vec::new();

        self.stream.read(&mut tmp)?;
        bytes.extend_from_slice(&tmp);
        self.handshake.decode_client_request1(bytes)
    }

    fn send_s0s1s2(&mut self) -> IOResult<()> {
        let s0s1s2 = self.handshake.get_s0s1s2();

        self.stream.write(s0s1s2.as_slice()).map(|_| ())
    }

    fn receive_c2(&mut self) -> IOResult<()> {
        let mut tmp: [u8; HANDSHAKE_CHUNK_SIZE] = [0; HANDSHAKE_CHUNK_SIZE];
        let mut bytes: Vec<u8> = Vec::new();

        self.stream.read(&mut tmp)?;
        bytes.extend_from_slice(&tmp);
        self.handshake.decode_client_request2(bytes)
    }

    fn handle_handshake(&mut self) -> IOResult<()> {
        self.receive_c0c1().and_then(
            |_| {
                self.state = RtmpState::ReceivedHandshake;
                self.send_s0s1s2().and_then(
                    |_| self.receive_c2().map(
                        |_| self.state = RtmpState::HandshakeDone
                    )
                )
            }
        ).map_err(
            |e| {
                error!("Client was rejected due to invalid handshake");
                e
            }
        )
    }

    fn insert_received_chunk(&mut self, chunk_id: ChunkId, inserted: LastChunk) {
        match self.received_chunks.get_mut(&chunk_id) {
            Some(last_chunk) => {
                last_chunk.set_message_type(inserted.get_message_type());
                last_chunk.set_message_id(inserted.get_message_id());
                last_chunk.set_message_len(inserted.get_message_len());
                last_chunk.set_timestamp(inserted.get_timestamp() - last_chunk.get_timestamp());
                last_chunk.set_chunk_data(inserted.get_chunk_data().clone());
            },
            _ => {
                self.received_chunks.insert(chunk_id, inserted);
            }
        }
    }

    fn insert_sent_chunk(&mut self, chunk_id: ChunkId, inserted: LastChunk) {
        match self.sent_chunks.get_mut(&chunk_id) {
            Some(last_chunk) => {
                last_chunk.set_message_type(inserted.get_message_type());
                last_chunk.set_message_id(inserted.get_message_id());
                last_chunk.set_message_len(inserted.get_message_len());
                last_chunk.set_timestamp(inserted.get_timestamp() - last_chunk.get_timestamp());
                last_chunk.set_chunk_data(inserted.get_chunk_data().clone());
            },
            _ => {
                self.sent_chunks.insert(chunk_id, inserted);
            }
        }
    }

    fn receive_chunk(&mut self) -> IOResult<Chunk> {
        let mut buffer = ByteBuffer::new(Vec::new());
        let mut bytes_basic_header: [u8; BasicHeader::LEN_ONE_BYTE] = [0; BasicHeader::LEN_ONE_BYTE];

        self.stream.read(&mut bytes_basic_header).map_err(
            |_| IOError::new(
                ErrorKind::InvalidInput,
                ChunkLengthError::new(
                    "The basic header couldn't be read.".to_string(),
                    None
                )
            )
        )?;

        let basic_header_type = bytes_basic_header[0] & 0x3f;

        buffer.put_bytes(bytes_basic_header.to_vec());

        match basic_header_type {
            0 => {
                let mut bytes_chunk_id: [u8; BasicHeader::LEN_ONE_BYTE] = [0; BasicHeader::LEN_ONE_BYTE];

                self.stream.read(&mut bytes_chunk_id).map_err(
                    |_| IOError::new(
                        ErrorKind::InvalidInput,
                        ChunkLengthError::new(
                            "The chunk id couldn't be read.".to_string(),
                            None
                        )
                    )
                )?;
                buffer.put_bytes(bytes_chunk_id.to_vec());
            },
            1 => {
                let mut bytes_chunk_id: [u8; BasicHeader::LEN_TWO_BYTES] = [0; BasicHeader::LEN_TWO_BYTES];

                self.stream.read(&mut bytes_chunk_id).map_err(
                    |_| IOError::new(
                        ErrorKind::InvalidInput,
                        ChunkLengthError::new(
                            "The chunk id couldn't be read.".to_string(),
                            None
                        )
                    )
                )?;
                buffer.put_bytes(bytes_chunk_id.to_vec());
            },
            _ => {}
        }

        let basic_header = buffer.decode_basic_header().ok_or(
            IOError::new(
                ErrorKind::InvalidData,
                ChunkFormatError::new(
                    "The basic header format is invalid.".to_string(),
                    None
                )
            )
        )?;
        println!("basic_header: {:x?}", basic_header);
        let message_format = basic_header.get_message_format();
        let chunk_id = basic_header.get_chunk_id();

        match message_format {
            MessageFormat::New => {
                let mut bytes_message_header: [u8; MessageHeader::LEN_NEW] = [0; MessageHeader::LEN_NEW];

                self.stream.read(&mut bytes_message_header).map_err(
                    |_| IOError::new(
                        ErrorKind::InvalidInput,
                        ChunkLengthError::new(
                            "The message header couldn't be read.".to_string(),
                            None
                        )
                    )
                )?;
                println!("bytes_message_header: {:x?}", bytes_message_header);
                buffer.put_bytes(bytes_message_header.to_vec());
            },
            MessageFormat::SameSource => {
                let mut bytes_message_header: [u8; MessageHeader::LEN_SAME_SOURCE] = [0; MessageHeader::LEN_SAME_SOURCE];

                self.stream.read(&mut bytes_message_header).map_err(
                    |_| IOError::new(
                        ErrorKind::InvalidInput,
                        ChunkLengthError::new(
                            "The message header couldn't be read.".to_string(),
                            None
                        )
                    )
                )?;
                println!("bytes_message_header: {:x?}", bytes_message_header);
                buffer.put_bytes(bytes_message_header.to_vec());
            },
            MessageFormat::TimerChange => {
                let mut bytes_message_header: [u8; MessageHeader::LEN_TIMER_CHANGE] = [0; MessageHeader::LEN_TIMER_CHANGE];

                self.stream.read(&mut bytes_message_header).map_err(
                    |_| IOError::new(
                        ErrorKind::InvalidInput,
                        ChunkLengthError::new(
                            "The message header couldn't be read.".to_string(),
                            None
                        )
                    )
                )?;
                println!("bytes_message_header: {:x?}", bytes_message_header);
                buffer.put_bytes(bytes_message_header.to_vec());
            },
            MessageFormat::Continue => {}
        }

        let message_header = buffer.decode_message_header(basic_header.get_message_format()).ok_or(
            IOError::new(
                ErrorKind::InvalidData,
                ChunkFormatError::new(
                    "The message header format is invalid.".to_string(),
                    None
                )
            )
        )?;
        println!("message_header: {:x?}", message_header);
        let timestamp = message_header.get_timestamp().or(
            self.received_chunks.get(&chunk_id).map(
                |last_chunk| last_chunk.get_timestamp()
            )
        ).unwrap();
        let message_len = message_header.get_message_len().or(
            self.received_chunks.get(&chunk_id).map(
                |last_chunk| last_chunk.get_message_len()
            )
        ).unwrap();
        let message_type = message_header.get_message_type().or(
            self.received_chunks.get(&chunk_id).map(
                |last_chunk| last_chunk.get_message_type()
            )
        ).unwrap();
        let message_id = message_header.get_message_id().or(
            self.received_chunks.get(&chunk_id).map(
                |last_chunk| last_chunk.get_message_id()
            )
        ).unwrap();

        if timestamp.as_secs() >= U24_MAX as u64 {
            let mut bytes_extended_timestamp: [u8; 4] = [0; 4];

            self.stream.read(&mut bytes_extended_timestamp).map_err(
                |_| IOError::new(
                    ErrorKind::InvalidInput,
                    ChunkLengthError::new(
                        "The extended timestamp couldn't be read.".to_string(),
                        None
                    )
                )
            )?;
            buffer.put_bytes(bytes_extended_timestamp.to_vec());
        }

        let extended_timestamp = buffer.decode_extended_timestamp(message_header);
        println!("extended_timestamp: {:x?}", extended_timestamp);
        let mut bytes_chunk_data: Vec<u8> = Vec::new();
        let mut remaining = message_len;

        while remaining > 0 {
            let capacity = min(self.chunk_size, remaining) as usize;
            let mut tmp: Vec<u8> = Vec::with_capacity(capacity);

            unsafe {
                tmp.set_len(capacity);
            }

            let read = self.stream.read(tmp.as_mut_slice()).map_err(
                |_| IOError::new(
                    ErrorKind::InvalidInput,
                    ChunkLengthError::new(
                        "The chunk data couldn't be read.".to_string(),
                        None
                    )
                )
            )?;

            bytes_chunk_data.append(&mut tmp);
            remaining -= read as u32;

            if read as u32 >= self.chunk_size {
                let mut skipped: [u8; 1] = [0; 1];

                self.stream.read(&mut skipped).map_err(
                    |_| IOError::new(
                        ErrorKind::InvalidInput,
                        ChunkLengthError::new(
                            "The chunk data couldn't be read.".to_string(),
                            None
                        )
                    )
                )?;
            }
        }

        // Disabled printing chunk data because the Audio/Video data is too long.
        // println!("bytes_chunk_data: {:x?}", bytes);
        buffer.put_bytes(bytes_chunk_data);

        let chunk_data = buffer.decode_chunk_data(message_type);
        let chunk = Chunk::new(
            basic_header,
            extended_timestamp,
            message_header,
            chunk_data.clone()
        );

        self.insert_received_chunk(
            chunk_id,
            LastChunk::new(
                message_type,
                message_id,
                message_len,
                extended_timestamp.unwrap_or(timestamp),
                chunk_data.unwrap()
            )
        );
        Ok(chunk)
    }

    fn send_chunk(&mut self, chunk_id: ChunkId, message_type: MessageType, message_id: u32, message_len: u32, timestamp: Duration, chunk_data: ChunkData) -> IOResult<()> {
        self.last_sent_chunk_id = chunk_id;

        let message_format = self.sent_chunks.get(&chunk_id).map_or(
            MessageFormat::New,
            |last_chunk| if last_chunk.get_message_id() == message_id {
                if last_chunk.get_message_type() == message_type && last_chunk.get_message_len() == message_len {
                    if last_chunk.get_timestamp() == timestamp {
                        MessageFormat::Continue
                    } else {
                        MessageFormat::TimerChange
                    }
                } else {
                    MessageFormat::SameSource
                }
            } else {
                MessageFormat::New
            }
        );
        let basic_header = BasicHeader::new(message_format, chunk_id);
        let message_header = match message_format {
            MessageFormat::New => MessageHeader::New { message_type, message_id, message_len, timestamp },
            MessageFormat::SameSource => MessageHeader::SameSource { message_type, message_len, timestamp },
            MessageFormat::TimerChange => MessageHeader::TimerChange { timestamp },
            MessageFormat::Continue => MessageHeader::Continue
        };
        let extended_timestamp = if timestamp.as_secs() >= U24_MAX as u64 {
            timestamp.checked_sub(Duration::from_secs(U24_MAX as u64))
        } else {
            None
        };
 
        let mut buffer = ByteBuffer::new(Vec::new());

        buffer.encode_basic_header(basic_header);
        buffer.encode_message_header(message_header);
        buffer.encode_extended_timestamp(extended_timestamp);

        let splits = message_len / self.chunk_size + ((message_len > self.chunk_size) && (message_len % self.chunk_size > 0)) as u32;

        if message_len > self.chunk_size {
            let mut chunk_data_buffer = ByteBuffer::new(Vec::new());

            chunk_data_buffer.encode_chunk_data(Some(chunk_data.clone()));

            let chunk_data_bytes = chunk_data_buffer.get_sliced_bytes(message_len as usize).unwrap();
            let basic_header = BasicHeader::new(MessageFormat::Continue, chunk_id);
            let message_header = MessageHeader::Continue;
            let continue_chunk = Chunk::new(basic_header, None, message_header, None);
            let continue_bytes = encode_chunk(continue_chunk);
            println!("continue: {:x?}", continue_bytes);
            let chunk_size = self.chunk_size as usize;
            let mut added: Vec<u8> = Vec::new();
            
            for i in 0..(splits as usize) {
                let start = chunk_size * i;
                let end = start + min(chunk_size, chunk_data_bytes[start..].len());

                if i > 0 {
                    added.extend_from_slice(continue_bytes.as_slice());
                }

                added.extend_from_slice(&chunk_data_bytes[start..end]);
            }

            buffer.put_bytes(added);
        } else {
            buffer.encode_chunk_data(Some(chunk_data.clone()));
        }

        println!("sent chunk: {:x?}", buffer.bytes());
        self.stream.write(buffer.bytes().as_slice()).map(
            |_| {
                self.insert_sent_chunk(
                    chunk_id,
                    LastChunk::new(
                        message_type,
                        message_id,
                        message_len,
                        timestamp,
                        chunk_data
                    )
                );
            }
        )
    }

    fn receive_chunk_size(&mut self, chunk_size: u32) -> IOResult<()> {
        if chunk_size & 0x80000000 == 1 {
            warn!("The most significant bit is 1 in chunk size.");
        }

        Ok(self.chunk_size = chunk_size)
    }

    fn send_chunk_size(&mut self) -> IOResult<()> {
        self.send_chunk(
            ChunkId::U8(Channel::Network as u8),
            MessageType::ChunkSize,
            0,
            4,
            Duration::default(),
            ChunkData::ChunkSize(self.chunk_size)
        )
    }

    fn receive_bytes_read(&mut self, bytes_read: u32) -> IOResult<()> {
        Ok(self.bytes_read += bytes_read)
    }

    fn send_bytes_read(&mut self) -> IOResult<()> {
        let mut timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_err(|e| IOError::new(ErrorKind::Other, e))?;

        timestamp -= self.received_chunks.get(&self.last_received_chunk_id).map_or(
            Duration::default(),
            |last_chunk| last_chunk.get_timestamp()
        );

        self.send_chunk(
            ChunkId::U8(Channel::Network as u8),
            MessageType::BytesRead,
            0,
            4,
            timestamp,
            ChunkData::BytesRead(self.bytes_read)
        )
    }

    fn send_ping_stream_begin(&mut self) -> IOResult<()> {
        let message_id = self.message_id;

        self.send_chunk(
            ChunkId::U8(Channel::Network as u8),
            MessageType::Ping,
            0,
            6,
            Duration::default(),
            ChunkData::Ping(PingData::StreamBegin(message_id))
        )
    }

    fn send_ping(&mut self, ping_type: PingType) -> IOResult<()> {
        use crate::messages::{
            PingType::*
        };

        match ping_type {
            StreamBegin => self.send_ping_stream_begin()
        }
    }

    fn receive_server_bandwidth(&mut self, bandwidth: u32) -> IOResult<()> {
        Ok(self.server_bandwidth = bandwidth)
    }

    fn send_server_bandwidth(&mut self) -> IOResult<()> {
        self.send_chunk(
            ChunkId::U8(Channel::Network as u8),
            MessageType::ServerBandwidth,
            0,
            4,
            Duration::default(),
            ChunkData::ServerBandwidth(self.server_bandwidth)
        )
    }

    fn receive_client_bandwidth(&mut self, bandwidth: u32, limit_type: LimitType) -> IOResult<()> {
        Ok({
            self.client_bandwidth = bandwidth;
            self.limit_type = limit_type;
        })
    }

    fn send_client_bandwidth(&mut self, limit_type: LimitType) -> IOResult<()> {
        self.send_chunk(
            ChunkId::U8(Channel::Network as u8),
            MessageType::ClientBandwidth,
            0,
            5,
            Duration::default(),
            ChunkData::ClientBandwidth(self.client_bandwidth, limit_type)
        )
    }

    fn receive_audio(&mut self, bytes: Vec<u8>) -> IOResult<()> {
        Ok(self.flv.append_audio(bytes))
    }

    fn receive_video(&mut self, bytes: Vec<u8>) -> IOResult<()> {
        Ok(self.flv.append_video(bytes))
    }

    fn receive_notify(&mut self, notify_command: NotifyCommand) -> IOResult<()> {
        use crate::messages::{
            NotifyCommand::*
        };

        match notify_command {
            SetDataFrame {
                data_frame,
                meta_data
            } => {
                if data_frame == "onMetaData" {
                    Ok(self.flv.append_meta_data(meta_data))
                } else {
                    Ok(println!("Unknown data frame: {}", data_frame))
                }
            },
            Unknown(bytes) => Ok(println!("Unknown notify command: {:x?}", bytes))
        }
    }

    fn receive_invoke(&mut self, invoke_command: InvokeCommand) -> IOResult<()> {
        use crate::messages::{
            InvokeCommand::*,
            NetConnectionCommand::*,
            NetStreamCommand::*,
            FcPublishCommand as fc,
        };
        use RtmpState::*;

        match invoke_command {
            NetConnection(net_connection_command) => match net_connection_command {
                Connect {
                    argument: _,
                    transaction_id,
                    command_object
                } => {
                    self.transaction_id = transaction_id;
                    self.command_object = command_object;
                    self.state = ReceivedConnect;
                },
                ReleaseStream {
                    transaction_id,
                    play_path
                } => {
                    self.transaction_id = transaction_id;
                    self.play_path = play_path;
                    self.state = ReceivedReleaseStream;
                },
                CreateStream {
                    transaction_id
                } => {
                    self.transaction_id = transaction_id;
                    self.state = ReceivedCreateStream;
                },
                _ => return Err(
                    IOError::new(
                        ErrorKind::InvalidInput,
                        ChunkFormatError::new(
                            "_result response has been passed when receiving.".to_string(),
                            None
                        )
                    )
                )
            },
            NetStream(net_stream_command) => match net_stream_command {
                Publish {
                    transaction_id,
                    play_path,
                    play_type
                } => {
                    self.transaction_id = transaction_id;
                    self.play_path = play_path;
                    self.play_type = play_type;
                    self.state = ReceivedPublish;
                },
                _ => return Err(
                    IOError::new(
                        ErrorKind::InvalidInput,
                        ChunkFormatError::new(
                            "onStatus response has been passed when receiving.".to_string(),
                            None
                        )
                    )
                )
            },
            FcPublish(fc_publish_command) => match fc_publish_command {
                fc::FcPublish {
                    transaction_id,
                    play_path
                } => {
                    self.transaction_id = transaction_id;
                    self.play_path = play_path;
                    self.state = ReceivedFcPublish;
                },
                _ => return Err(
                    IOError::new(
                        ErrorKind::InvalidInput,
                        ChunkFormatError::new(
                            "onFCPublish response has been passed when receiving.".to_string(),
                            None
                        )
                    )
                )
            },
            Unknown(bytes) => {
                println!("unknown invoke command: {:x?}", bytes);
            }
        }

        Ok(())
    }

    fn send_connect_result(&mut self, transaction_id: u64) -> IOResult<()> {
        let result = NetConnectionResult::Result;
        let mut properties = HashMap::new();

        properties.insert("fmsVer".to_string(), AmfData::String("FMS/5.0.15".to_string()));
        properties.insert("capabilities".to_string(), AmfData::Number(31f64));

        let mut information = HashMap::new();

        information.insert("level".to_string(), AmfData::String("status".to_string()));
        information.insert("code".to_string(), AmfData::String("NetConnection.Connect.Success".to_string()));
        information.insert("description".to_string(), AmfData::String("Connection succeeded.".to_string()));
        information.insert("objectEncoding".to_string(), AmfData::Number(ObjectEncoding::Amf0 as u8 as f64));

        let mut buffer = ByteBuffer::new(Vec::new());
        let chunk_data = ChunkData::Invoke(
            InvokeCommand::NetConnection(
                NetConnectionCommand::ConnectResult {
                    result,
                    transaction_id,
                    properties,
                    information: information.into()
                }
            )
        );

        buffer.encode_chunk_data(Some(chunk_data.clone()));

        let message_len = buffer.len() as u32;

        self.send_chunk(
            ChunkId::U8(Channel::System as u8),
            MessageType::Invoke,
            0,
            message_len,
            Duration::default(),
            chunk_data
        )
    }

    fn handle_connect(&mut self, transaction_id: u64) -> IOResult<()> {
        self.send_connect_result(transaction_id).and_then(
            |_| self.send_server_bandwidth().and_then(
                |_| self.send_client_bandwidth(LimitType::Dynamic).and_then(
                    |_| self.send_ping(PingType::StreamBegin).and_then(
                        |_| self.send_chunk_size().and_then(
                            |_| self.send_connect_result(transaction_id).map(
                                |_| self.state = RtmpState::SentConnectResult
                            )
                        )
                    )
                )
            )
        )
    }

    fn handle_release_stream(&mut self, transaction_id: u64) -> IOResult<()> {
        let result = NetConnectionResult::Result;
        let chunk_data = ChunkData::Invoke(
            InvokeCommand::NetConnection(
                NetConnectionCommand::ReleaseStreamResult {
                    result,
                    transaction_id
                }
            )
        );
        let mut buffer = ByteBuffer::new(Vec::new());

        buffer.encode_chunk_data(Some(chunk_data.clone()));

        let message_len = buffer.len() as u32;

        self.send_chunk(
            ChunkId::U8(Channel::System as u8),
            MessageType::Invoke,
            0,
            message_len,
            Duration::default(),
            chunk_data
        ).map(|_| self.state = RtmpState::SentReleaseStreamResult)
    }

    fn handle_fc_publish(&mut self) -> IOResult<()> {
        let chunk_data = ChunkData::Invoke(
            InvokeCommand::FcPublish(
                FcPublishCommand::OnFcPublish
            )
        );
        let mut buffer = ByteBuffer::new(Vec::new());

        buffer.encode_chunk_data(Some(chunk_data.clone()));

        let message_len = buffer.len() as u32;

        self.send_chunk(
            ChunkId::U8(Channel::System as u8),
            MessageType::Invoke,
            0,
            message_len,
            Duration::default(),
            chunk_data
        ).map(|_| self.state = RtmpState::SentOnFcPublish)
    }

    fn handle_create_stream(&mut self, transaction_id: u64, message_id: u32) -> IOResult<()> {
        let result = NetConnectionResult::Result;
        let chunk_data = ChunkData::Invoke(
            InvokeCommand::NetConnection(
                NetConnectionCommand::CreateStreamResult {
                    result,
                    message_id,
                    transaction_id
                }
            )
        );
        let mut buffer = ByteBuffer::new(Vec::new());

        buffer.encode_chunk_data(Some(chunk_data.clone()));

        let message_len = buffer.len() as u32;

        self.send_chunk(
            ChunkId::U8(Channel::System as u8),
            MessageType::Invoke,
            0,
            message_len,
            Duration::default(),
            chunk_data
        ).map(|_| self.state = RtmpState::SentCreateStreamResult)
    }

    fn send_on_status(&mut self, status: Status, play_path: String) -> IOResult<()> {
        let mut m: HashMap<String, AmfData> = HashMap::new();

        m.insert("level".to_string(), AmfData::String("status".to_string()));
        m.insert("code".to_string(), AmfData::String(status.into()));
        m.insert("description".to_string(), AmfData::String(format!("{} is now published", play_path)));
        m.insert("details".to_string(), AmfData::String(play_path));

        let chunk_data = ChunkData::Invoke(
            InvokeCommand::NetStream(
                NetStreamCommand::OnStatus {
                    transaction_id: 0,
                    info_object: m.into()
                }
            )
        );
        let mut buffer = ByteBuffer::new(Vec::new());

        buffer.encode_chunk_data(Some(chunk_data.clone()));

        let message_len = buffer.len() as u32;

        self.send_chunk(
            ChunkId::U8(Channel::System as u8),
            MessageType::Invoke,
            0,
            message_len,
            Duration::default(),
            chunk_data
        )
    }

    fn handle_publish(&mut self) -> IOResult<()> {
        self.send_ping(PingType::StreamBegin).and_then(
            |_| {
                let play_path = self.play_path.clone();

                self.send_on_status(
                    Status::NetStream(NetStreamStatus::Publish(PublishStatus::Start)),
                    play_path
                ).map(|_| self.state = RtmpState::Connected)
            }
        )
    }

    fn receive_unknown(&mut self, _: Vec<u8>) -> IOResult<()> {
        panic!("Stop at here!")
    }

    pub(crate) fn handle_chunk(&mut self) -> IOResult<()> {
        use RtmpState::*;
        use crate::messages::ChunkData::*;

        match self.state {
            TcpConnect => self.handle_handshake(),
            Disconnecting | Disconnected | Error => unimplemented!("when disconnection and error"),
            _ => {
                let received = self.receive_chunk()?;

                match received.get_chunk_data().clone() {
                    Some(chunk_data) => match chunk_data {
                        ChunkSize(chunk_size) => self.receive_chunk_size(chunk_size),
                        BytesRead(bytes_read) => self.receive_bytes_read(bytes_read),
                        Ping(_) => unimplemented!("when received ping"),
                        ServerBandwidth(bandwidth) => self.receive_server_bandwidth(bandwidth),
                        ClientBandwidth(bandwidth, limit_type) => self.receive_client_bandwidth(bandwidth, limit_type),
                        Audio(bytes) => self.receive_audio(bytes),
                        Video(bytes) => self.receive_video(bytes),
                        Notify(notify_command) => self.receive_notify(notify_command),
                        Invoke(invoke_command) => {
                            self.receive_invoke(invoke_command)?;
                            println!("state: {:?}", self.state);

                            let transaction_id = self.transaction_id;

                            if let ReceivedConnect = self.state {
                                self.handle_connect(transaction_id)
                            } else if let ReceivedReleaseStream = self.state {
                                self.handle_release_stream(transaction_id)
                            } else if let ReceivedFcPublish = self.state {
                                self.handle_fc_publish()
                            } else if let ReceivedCreateStream = self.state {
                                let message_id = self.message_id;

                                self.handle_create_stream(transaction_id, message_id)
                            } else if let ReceivedPublish = self.state {
                                self.handle_publish()
                            } else {
                                unimplemented!("other commands")
                            }
                        },
                        Unknown(bytes) => self.receive_unknown(bytes)
                    },
                    _ => Err(
                        IOError::new(
                            ErrorKind::InvalidInput,
                            ChunkFormatError::new(
                                "Somehow the chunk data is nothing.".to_string(),
                                None
                            )
                        )
                    )
                }
            }
        }
    }
}
