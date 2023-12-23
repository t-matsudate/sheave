mod encryption_algorithm;
mod handshake;
mod basic_header;
mod message_header;
mod extended_timestamp;
mod chunk_data;

use std::{
    future::Future,
    io::Result as IOResult,
    pin::{
        Pin,
        pin
    },
    task::{
        Context as FutureContext,
        Poll
    },
    time::Duration
};
use futures::{
    future::{
        PollFn,
        poll_fn
    },
    ready
};
use tokio::io::AsyncWrite;
use crate::{
    ByteBuffer,
    Encoder,
    handlers::{
        LastChunk,
        RtmpContext
    },
    messages::{
        ChunkData,
        headers::{
            BasicHeader,
            MessageFormat,
            MessageHeader
        }
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

pub fn write_chunk<'a, W, T>(mut writer: Pin<&'a mut W>, rtmp_context: &'a mut RtmpContext, mut timestamp: Duration, message_id: u32, data: &'a T) -> PollFn<Box<dyn FnMut(&mut FutureContext) -> Poll<IOResult<()>> + 'a>>
where
    W: AsyncWrite,
    T: ChunkData,
    ByteBuffer: Encoder<T>
{
    poll_fn(Box::new(move |cx| {
        let mut buffer = ByteBuffer::default();
        buffer.encode(data);
        let data_bytes: Vec<u8> = buffer.into();

        let message_format = match rtmp_context.get_last_sent_chunk(&(T::CHANNEL as u16)) {
            Some(last_sent_chunk) => if last_sent_chunk.get_message_id() != message_id {
                MessageFormat::New
            } else if last_sent_chunk.get_message_length() != data_bytes.len() as u32 || last_sent_chunk.get_message_type() != T::MESSAGE_TYPE {
                MessageFormat::SameSource
            } else {
                MessageFormat::TimerChange
            },
            _ => MessageFormat::New
        };
        let extended_timestamp = if MessageHeader::U24_MAX <= timestamp.as_millis() as u32 {
            let extended_timestamp = timestamp;
            timestamp = Duration::from_millis(MessageHeader::U24_MAX as u64);
            Some(extended_timestamp)
        } else {
            None
        };
        let basic_header = BasicHeader::new(message_format, T::CHANNEL as u16);
        let message_header;
        match message_format {
            MessageFormat::New => message_header = MessageHeader::New((timestamp, data_bytes.len() as u32, T::MESSAGE_TYPE, message_id).into()),
            MessageFormat::SameSource => message_header = MessageHeader::SameSource((timestamp, data_bytes.len() as u32, T::MESSAGE_TYPE).into()),
            MessageFormat::TimerChange => message_header = MessageHeader::TimerChange(timestamp.into()),
            _ => unreachable!("The continue header never appears in this step.")
        };
        
        ready!(pin!(write_basic_header(writer.as_mut(), &basic_header)).poll(cx))?;
        ready!(pin!(write_message_header(writer.as_mut(), &message_header)).poll(cx))?;
        if let Some(extended_timestamp) = extended_timestamp {
            ready!(pin!(write_extended_timestamp(writer.as_mut(), extended_timestamp)).poll(cx))?;
        }
        ready!(
            pin!(
                write_chunk_data(
                    writer.as_mut(),
                    T::CHANNEL as u16,
                    rtmp_context.get_sending_chunk_size(),
                    &data_bytes
                )
            ).poll(cx)
        )?;

        if let Some(last_sent_chunk) = rtmp_context.get_last_sent_chunk_mut(&(T::CHANNEL as u16)) {
            last_sent_chunk.update(&message_header, extended_timestamp);
        } else {
            rtmp_context.insert_sent_chunk(
                T::CHANNEL as u16,
                LastChunk::new(message_header)
            );
        }

        Poll::Ready(Ok(()))
    }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::{
        Decoder,
        handlers::{
            StreamWrapper,
            VecStream
        },
        messages::Connect,
        readers::*
    };
    use super::*;

    #[tokio::test]
    async fn write_connect() {
        let stream = Arc::new(StreamWrapper::new(VecStream::default()));
        let rtmp_context = Arc::new(RtmpContext::default());
        let expected = Connect::default();

        let result = write_chunk(stream.make_weak_pin(), rtmp_context.make_weak_mut(), Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.make_weak_pin()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.make_weak_pin(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.make_weak_pin(), chunk_size, message_length).await.unwrap();
        assert_eq!(Connect::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(Connect::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<Connect>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }
}
