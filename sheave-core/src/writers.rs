mod encryption_algorithm;
mod handshake;
mod basic_header;
mod message_header;
mod extended_timestamp;
mod chunk_data;

use std::{
    io::Result as IOResult,
    pin::Pin,
    time::Duration
};
use tokio::io::AsyncWrite;
use crate::{
    U24_MAX,
    handlers::{
        LastChunk,
        RtmpContext
    },
    messages::headers::{
        BasicHeader,
        MessageFormat,
        MessageHeader,
        MessageType
    }
};
pub use self::{
    encryption_algorithm::*,
    handshake::*,
    basic_header::*,
    message_header::*,
    extended_timestamp::*,
    chunk_data::*
};

/// A wrapper for writing a chunk into streams.
///
/// The RTMP needs to refer previous states for deciding sending chunk pattern.
/// But to check them in every step is troublesome and also can make some bug.
/// This reduces their risks.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::{
///         Pin,
///         pin
///     },
///     time::Duration
/// };
/// use sheave_core::{
///     ByteBuffer,
///     Encoder,
///     handlers::{
///         RtmpContext,
///         VecStream
///     },
///     messages::{
///         ChunkData,
///         Connect,
///         amf::v0::Object
///     },
///     readers::{
///         read_basic_header,
///         read_message_header,
///         read_chunk_data
///     },
///     writers::write_chunk
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let mut stream = pin!(VecStream::default());
///     let mut rtmp_context = RtmpContext::default();
///     let mut buffer = ByteBuffer::default();
///     buffer.encode(&Connect::new(Object::default()));
///     let expected: Vec<u8> = buffer.into();
///     write_chunk(stream.as_mut(), &mut rtmp_context, Connect::CHANNEL.into(), Duration::default(), Connect::MESSAGE_TYPE, u32::default(), &expected).await?;
///
///     let basic_header = read_basic_header(stream.as_mut()).await?;
///     let message_header = read_message_header(stream.as_mut(), basic_header.get_message_format()).await?;
///     let receiving_chunk_size = rtmp_context.get_receiving_chunk_size();
///     let message_length = rtmp_context.get_last_sent_chunk(&basic_header.get_chunk_id()).unwrap().get_message_length();
///     let actual = read_chunk_data(stream.as_mut(), receiving_chunk_size, message_length).await?;
///     assert_eq!(expected, actual);
///
///     Ok(())
/// }
/// ```
pub async fn write_chunk<'a, W: AsyncWrite>(mut writer: Pin<&'a mut W>, rtmp_context: &'a mut RtmpContext, chunk_id: u16, mut timestamp: Duration, message_type: MessageType, message_id: u32, data: &'a [u8]) -> IOResult<()> {
    let message_format = if let Some(last_sent_chunk) = rtmp_context.get_last_sent_chunk(&chunk_id) {
        if message_id != last_sent_chunk.get_message_id() {
            MessageFormat::New
        } else if (message_type != last_sent_chunk.get_message_type()) || (data.len() != last_sent_chunk.get_message_length() as usize) {
            MessageFormat::SameSource
        } else if timestamp != last_sent_chunk.get_timestamp() {
            MessageFormat::TimerChange
        } else {
            MessageFormat::Continue
        }
    } else {
        MessageFormat::New
    };
    let extended_timestamp = if timestamp.as_millis() >= U24_MAX as u128 {
        let extended_timestamp = Some(timestamp);
        timestamp = Duration::from_millis(U24_MAX as u64);
        extended_timestamp
    } else {
        None
    };
    let message_header = match message_format {
        MessageFormat::New => MessageHeader::New((timestamp, data.len() as u32, message_type, message_id).into()),
        MessageFormat::SameSource => MessageHeader::SameSource((timestamp, data.len() as u32, message_type).into()),
        MessageFormat::TimerChange => MessageHeader::TimerChange(timestamp.into()),
        MessageFormat::Continue => MessageHeader::Continue
    };

    write_basic_header(writer.as_mut(), &BasicHeader::new(message_format, chunk_id)).await?;
    write_message_header(writer.as_mut(), &message_header).await?;
    if let Some(extended_timestamp) = extended_timestamp {
        write_extended_timestamp(writer.as_mut(), extended_timestamp).await?;
    }

    if let Some(last_sent_chunk) = rtmp_context.get_last_sent_chunk_mut(&chunk_id) {
        if let Some(extended_timestamp) = extended_timestamp {
            last_sent_chunk.set_timestamp(extended_timestamp);
        } else {
            message_header.get_timestamp().map(
                |timestamp| last_sent_chunk.set_timestamp(timestamp)
            );
        }
        message_header.get_message_length().map(
            |message_length| last_sent_chunk.set_message_length(message_length)
        );
        message_header.get_message_type().map(
            |message_type| last_sent_chunk.set_message_type(message_type)
        );
        message_header.get_message_id().map(
            |message_id| last_sent_chunk.set_message_id(message_id)
        );
    } else {
        rtmp_context.insert_sent_chunk(
            chunk_id,
            LastChunk::new(
                message_header.get_timestamp().unwrap(),
                message_header.get_message_length().unwrap(),
                message_header.get_message_type().unwrap(),
                message_header.get_message_id().unwrap()
            )
        );
    }

    write_chunk_data(writer.as_mut(), chunk_id, rtmp_context.get_sending_chunk_size(), data).await
}
