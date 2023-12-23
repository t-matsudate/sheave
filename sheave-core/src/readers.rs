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
    }
};
use futures::{
    future::{
        PollFn,
        poll_fn
    },
    ready
};
use tokio::io::AsyncRead;
use crate::{
    ByteBuffer,
    Decoder,
    handlers::{
        LastChunk,
        RtmpContext
    },
    messages::{
        ChunkData,
        headers::MessageHeader
    },
};
pub use self::{
    encryption_algorithm::*,
    handshake::*,
    basic_header::*,
    message_header::*,
    extended_timestamp::*,
    chunk_data::*
};

pub fn read_chunk<'a, R, T>(mut reader: Pin<&'a mut R>, rtmp_context: &'a mut RtmpContext) -> PollFn<Box<dyn FnMut(&mut FutureContext) -> Poll<IOResult<T>> + 'a>>
where
    R: AsyncRead,
    T: ChunkData,
    ByteBuffer: Decoder<T>
{
    poll_fn(Box::new(move |cx| {
        let basic_header = ready!(pin!(read_basic_header(reader.as_mut())).poll(cx))?;
        let chunk_id = basic_header.get_chunk_id();
        let message_header = ready!(pin!(read_message_header(reader.as_mut(), basic_header.get_message_format())).poll(cx))?;
        let extended_timestamp = if let Some(timestamp) = message_header.get_timestamp() {
            if MessageHeader::U24_MAX == timestamp.as_millis() as u32 {
                let extended_timestamp = ready!(pin!(read_extended_timestamp(reader.as_mut())).poll(cx))?;
                Some(extended_timestamp)
            } else {
                None
            }
        } else {
            None
        };
        let message_length = if let Some(message_length) = message_header.get_message_length() {
            message_length
        } else {
            rtmp_context.get_last_received_chunk(&chunk_id).unwrap().get_message_length()
        };
        let data = ready!(pin!(read_chunk_data(reader.as_mut(), rtmp_context.get_receiving_chunk_size(), message_length)).poll(cx))?;
        let decoded = Decoder::<T>::decode(&mut ByteBuffer::from(data))?;

        if let Some(last_received_chunk) = rtmp_context.get_last_received_chunk_mut(&chunk_id) {
            last_received_chunk.update(&message_header, extended_timestamp);
        } else {
            // It is the 11 bytes type if the chunk stream is just created.
            rtmp_context.insert_received_chunk(
                chunk_id,
                LastChunk::new(message_header)
            );
        };

        Poll::Ready(Ok(decoded))
    }))
}

#[cfg(test)]
mod tests {
    use std::{
        sync::Arc,
        time::Duration
    };
    use crate::{
        Encoder,
        handlers::{
            StreamWrapper,
            VecStream
        },
        messages::{
            Connect,
            headers::{
                BasicHeader,
                MessageFormat
            }
        },
        writers::*
    };
    use super::*;

    #[tokio::test]
    async fn read_connect() {
        let mut buffer = ByteBuffer::default();
        let expected = Connect::default();
        buffer.encode(&expected);
        let data: Vec<u8> = buffer.into();

        let stream = Arc::new(StreamWrapper::new(VecStream::default()));
        let mut rtmp_context = RtmpContext::default();
        write_basic_header(
            stream.make_weak_pin(),
            &BasicHeader::new(MessageFormat::New, Connect::CHANNEL as u16)
        ).await.unwrap();
        write_message_header(
            stream.make_weak_pin(),
            &MessageHeader::New((Duration::default(), data.len() as u32, Connect::MESSAGE_TYPE, u32::default()).into())
        ).await.unwrap();
        write_chunk_data(
            stream.make_weak_pin(),
            Connect::CHANNEL as u16,
            rtmp_context.get_sending_chunk_size(),
            &data
        ).await.unwrap();

        let result: IOResult<Connect> = read_chunk(stream.make_weak_pin(), &mut rtmp_context).await;
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(expected, actual)
    }
}
