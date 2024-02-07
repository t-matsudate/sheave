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

/// Reads a chunk from streams.
///
/// # Errors
///
/// This will be occured several errors in decoding.
/// For examples:
///
/// * When streams didn't have enough data.
/// * When data format is invalid.
/// * When something value in data differed from what's expected.
///
/// Because this is expected receiving chunk data is correctly ready in streams.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::Result as IOResult,
///     pin::pin,
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
///         ChunkSize,
///         Command,
///         Connect,
///         headers::{
///             BasicHeader,
///             MessageFormat,
///             MessageHeader
///         }
///     },
///     readers::read_chunk,
///     writers::{
///         write_basic_header,
///         write_chunk_data,
///         write_message_header
///     }
/// };
///
/// #[tokio::main]
/// async fn main() -> IOResult<()> {
///     let mut buffer = ByteBuffer::default();
///     buffer.encode(&Connect::default());
///     let data: Vec<u8> = buffer.into();
///     let mut stream = pin!(VecStream::default());
///     write_basic_header(stream.as_mut(), &BasicHeader::new(MessageFormat::New, Connect::CHANNEL as u16)).await?;
///     write_message_header(stream.as_mut(), &MessageHeader::New((Duration::default(), data.len() as u32, Connect::MESSAGE_TYPE, u32::default()).into())).await?;
///     write_chunk_data(stream.as_mut(), Connect::CHANNEL as u16, ChunkSize::default(), &data).await?;
///     let result: IOResult<Connect> = read_chunk(stream.as_mut(), &mut RtmpContext::default()).await;
///     assert!(result.is_ok());
///
///     let chunk = result.unwrap();
///     assert_eq!("connect", chunk.get_command_name());
///
///     Ok(())
/// }
/// ```
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
    use std::time::Duration;
    use crate::{
        Encoder,
        handlers::VecStream,
        messages::{
            Connect,
            ConnectResult,
            ReleaseStream,
            ReleaseStreamResult,
            FcPublish,
            OnFcPublish,
            CreateStream,
            CreateStreamResult,
            Publish,
            amf::v0::Number,
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

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        write_basic_header(
            stream.as_mut(),
            &BasicHeader::new(MessageFormat::New, Connect::CHANNEL as u16)
        ).await.unwrap();
        write_message_header(
            stream.as_mut(),
            &MessageHeader::New((Duration::default(), data.len() as u32, Connect::MESSAGE_TYPE, u32::default()).into()
            )
        ).await.unwrap();
        write_chunk_data(
            stream.as_mut(),
            Connect::CHANNEL as u16,
            rtmp_context.get_sending_chunk_size(),
            &data
        ).await.unwrap();

        let result: IOResult<Connect> = read_chunk(stream.as_mut(), &mut rtmp_context).await;
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn read_connect_result() {
        let mut buffer = ByteBuffer::default();
        let expected = ConnectResult::default();
        buffer.encode(&expected);
        let data: Vec<u8> = buffer.into();

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        write_basic_header(
            stream.as_mut(),
            &BasicHeader::new(MessageFormat::New, ConnectResult::CHANNEL as u16)
        ).await.unwrap();
        write_message_header(
            stream.as_mut(),
            &MessageHeader::New((Duration::default(), data.len() as u32, ConnectResult::MESSAGE_TYPE, u32::default()).into())
        ).await.unwrap();
        write_chunk_data(
            stream.as_mut(),
            ConnectResult::CHANNEL as u16,
            rtmp_context.get_sending_chunk_size(),
            &data
        ).await.unwrap();

        let result: IOResult<ConnectResult> = read_chunk(stream.as_mut(), &mut rtmp_context).await;
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn read_release_stream() {
        let mut buffer = ByteBuffer::default();
        let expected = ReleaseStream::new(2u8.into(), "".into());
        buffer.encode(&expected);
        let data: Vec<u8> = buffer.into();

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        write_basic_header(
            stream.as_mut(),
            &BasicHeader::new(MessageFormat::New, ReleaseStream::CHANNEL as u16)
        ).await.unwrap();
        write_message_header(
            stream.as_mut(),
            &MessageHeader::New((Duration::default(), data.len() as u32, ReleaseStream::MESSAGE_TYPE, u32::default()).into())
        ).await.unwrap();
        write_chunk_data(
            stream.as_mut(),
            ReleaseStream::CHANNEL as u16,
            rtmp_context.get_sending_chunk_size(),
            &data
        ).await.unwrap();

        let result: IOResult<ReleaseStream> = read_chunk(stream.as_mut(), &mut rtmp_context).await;
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn read_release_stream_result() {
        let mut buffer = ByteBuffer::default();
        let expected = ReleaseStreamResult::new("_result".into(), 2u8.into());
        buffer.encode(&expected);
        let data: Vec<u8> = buffer.into();

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        write_basic_header(
            stream.as_mut(),
            &BasicHeader::new(MessageFormat::New, ReleaseStreamResult::CHANNEL as u16)
        ).await.unwrap();
        write_message_header(
            stream.as_mut(),
            &MessageHeader::New((Duration::default(), data.len() as u32, ReleaseStreamResult::MESSAGE_TYPE, u32::default()).into())
        ).await.unwrap();
        write_chunk_data(
            stream.as_mut(),
            ReleaseStreamResult::CHANNEL as u16,
            rtmp_context.get_sending_chunk_size(),
            &data
        ).await.unwrap();

        let result: IOResult<ReleaseStreamResult> = read_chunk(stream.as_mut(), &mut rtmp_context).await;
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn read_fc_publish() {
        let mut buffer = ByteBuffer::default();
        let expected = FcPublish::new(3u8.into(), "".into());
        buffer.encode(&expected);
        let data: Vec<u8> = buffer.into();

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        write_basic_header(
            stream.as_mut(),
            &BasicHeader::new(MessageFormat::New, FcPublish::CHANNEL as u16)
        ).await.unwrap();
        write_message_header(
            stream.as_mut(),
            &MessageHeader::New((Duration::default(), data.len() as u32, FcPublish::MESSAGE_TYPE, u32::default()).into())
        ).await.unwrap();
        write_chunk_data(
            stream.as_mut(),
            FcPublish::CHANNEL as u16,
            rtmp_context.get_sending_chunk_size(),
            &data
        ).await.unwrap();

        let result: IOResult<FcPublish> = read_chunk(stream.as_mut(), &mut rtmp_context).await;
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn read_on_fc_publish() {
        let mut buffer = ByteBuffer::default();
        let expected = OnFcPublish;
        buffer.encode(&expected);
        let data: Vec<u8> = buffer.into();

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        write_basic_header(
            stream.as_mut(),
            &BasicHeader::new(MessageFormat::New, OnFcPublish::CHANNEL as u16)
        ).await.unwrap();
        write_message_header(
            stream.as_mut(),
            &MessageHeader::New((Duration::default(), data.len() as u32, OnFcPublish::MESSAGE_TYPE, u32::default()).into())
        ).await.unwrap();
        write_chunk_data(
            stream.as_mut(),
            OnFcPublish::CHANNEL as u16,
            rtmp_context.get_sending_chunk_size(),
            &data
        ).await.unwrap();

        let result: IOResult<OnFcPublish> = read_chunk(stream.as_mut(), &mut rtmp_context).await;
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn read_create_stream() {
        let mut buffer = ByteBuffer::default();
        let expected = CreateStream::new(4u8.into());
        buffer.encode(&expected);
        let data: Vec<u8> = buffer.into();

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        write_basic_header(
            stream.as_mut(),
            &BasicHeader::new(MessageFormat::New, CreateStream::CHANNEL as u16)
        ).await.unwrap();
        write_message_header(
            stream.as_mut(),
            &MessageHeader::New((Duration::default(), data.len() as u32, CreateStream::MESSAGE_TYPE, u32::default()).into())
        ).await.unwrap();
        write_chunk_data(
            stream.as_mut(),
            CreateStream::CHANNEL as u16,
            rtmp_context.get_sending_chunk_size(),
            &data
        ).await.unwrap();

        let result: IOResult<CreateStream> = read_chunk(stream.as_mut(), &mut rtmp_context).await;
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn read_create_stream_result() {
        let mut buffer = ByteBuffer::default();
        let expected = CreateStreamResult::new("_result".into(), 4u8.into(), Number::default());
        buffer.encode(&expected);
        let data: Vec<u8> = buffer.into();

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        write_basic_header(
            stream.as_mut(),
            &BasicHeader::new(MessageFormat::New, CreateStreamResult::CHANNEL as u16)
        ).await.unwrap();
        write_message_header(
            stream.as_mut(),
            &MessageHeader::New((Duration::default(), data.len() as u32, CreateStreamResult::MESSAGE_TYPE, u32::default()).into())
        ).await.unwrap();
        write_chunk_data(
            stream.as_mut(),
            CreateStreamResult::CHANNEL as u16,
            rtmp_context.get_sending_chunk_size(),
            &data
        ).await.unwrap();

        let result: IOResult<CreateStreamResult> = read_chunk(stream.as_mut(), &mut rtmp_context).await;
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn read_publish() {
        let mut buffer = ByteBuffer::default();
        let expected = Publish::new(5u8.into(), "".into(), "live".into());
        buffer.encode(&expected);
        let data: Vec<u8> = buffer.into();

        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        write_basic_header(
            stream.as_mut(),
            &BasicHeader::new(MessageFormat::New, Publish::CHANNEL as u16)
        ).await.unwrap();
        write_message_header(
            stream.as_mut(),
            &MessageHeader::New((Duration::default(), data.len() as u32, Publish::MESSAGE_TYPE, u32::default()).into())
        ).await.unwrap();
        write_chunk_data(
            stream.as_mut(),
            Publish::CHANNEL as u16,
            rtmp_context.get_sending_chunk_size(),
            &data
        ).await.unwrap();

        let result: IOResult<Publish> = read_chunk(stream.as_mut(), &mut rtmp_context).await;
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(expected, actual)
    }
}
