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

/// Writes a chunk into streams.
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
///     Decoder,
///     handlers::{
///         RtmpContext,
///         VecStream
///     },
///     messages::{
///         ChunkData,
///         ChunkSize,
///         Command,
///         Connect
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
///
///     write_chunk(stream.as_mut(), &mut RtmpContext::default(), Duration::default(), u32::default(), &Connect::default()).await?;
///     let basic_header = read_basic_header(stream.as_mut()).await?;
///     let message_header = read_message_header(stream.as_mut(), basic_header.get_message_format()).await?;
///     let data = read_chunk_data(stream.as_mut(), ChunkSize::default(), message_header.get_message_length().unwrap()).await?;
///     let mut buffer: ByteBuffer = data.into();
///     let chunk: Connect = buffer.decode()?;
///     assert_eq!(Connect::CHANNEL as u16, basic_header.get_chunk_id());
///     assert_eq!(Connect::MESSAGE_TYPE, message_header.get_message_type().unwrap());
///     assert_eq!("connect", chunk.get_command_name());
///
///     Ok(())
/// }
/// ```
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
    use crate::{
        Decoder,
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
            StreamBegin,
            OnStatus,
            SetDataFrame,
            amf::v0::Number
        },
        readers::*
    };
    use super::*;

    #[tokio::test]
    async fn write_connect() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = Connect::default();

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(Connect::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(Connect::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<Connect>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_connect_result() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = ConnectResult::default();

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(ConnectResult::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(ConnectResult::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<ConnectResult>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_release_stream() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = ReleaseStream::new(2u8.into(), "".into());

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(ReleaseStream::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(ReleaseStream::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<ReleaseStream>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_release_stream_result() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = ReleaseStreamResult::new("_result".into(), 2u8.into());

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(ReleaseStreamResult::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(ReleaseStreamResult::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<ReleaseStreamResult>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_fc_publish() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = FcPublish::new(2u8.into(), "".into());

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(FcPublish::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(FcPublish::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<FcPublish>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_on_fc_publish() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = OnFcPublish;

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(OnFcPublish::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(OnFcPublish::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<OnFcPublish>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_create_stream() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = CreateStream::new(4u8.into());

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(CreateStream::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(CreateStream::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<CreateStream>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_create_stream_result() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = CreateStreamResult::new("_result".into(), 4u8.into(), Number::default());

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(CreateStreamResult::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(CreateStreamResult::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<CreateStreamResult>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_publish() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = Publish::new(5u8.into(), "".into(), "live".into());

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(Publish::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(Publish::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<Publish>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_stream_begin() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = StreamBegin::new(u32::default());

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(StreamBegin::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(StreamBegin::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<StreamBegin>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_on_status() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = OnStatus::default();

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(OnStatus::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(OnStatus::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<OnStatus>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn write_set_data_frame() {
        let mut stream = pin!(VecStream::default());
        let mut rtmp_context = RtmpContext::default();
        let expected = SetDataFrame::default();

        let result = write_chunk(stream.as_mut(), &mut rtmp_context, Duration::default(), u32::default(), &expected).await;
        assert!(result.is_ok());
        let basic_header = read_basic_header(stream.as_mut()).await.unwrap();
        let message_format = basic_header.get_message_format();
        let message_header = read_message_header(stream.as_mut(), message_format).await.unwrap();
        let message_length = message_header.get_message_length().unwrap();
        let chunk_size = rtmp_context.get_receiving_chunk_size();
        let data = read_chunk_data(stream.as_mut(), chunk_size, message_length).await.unwrap();
        assert_eq!(SetDataFrame::CHANNEL as u16, basic_header.get_chunk_id());
        assert_eq!(MessageFormat::New, message_format);
        assert_eq!(Duration::default(), message_header.get_timestamp().unwrap());
        assert_eq!(message_length, data.len() as u32);
        assert_eq!(SetDataFrame::MESSAGE_TYPE, message_header.get_message_type().unwrap());
        assert_eq!(u32::default(), message_header.get_message_id().unwrap());

        let actual = Decoder::<SetDataFrame>::decode(&mut ByteBuffer::from(data)).unwrap();
        assert_eq!(expected, actual)
    }
}
