//! # The traits to encode the chunk messages.
//!
//! First, we will receive the chunk as bytes (that is, `Vec<u8>`).
//! Therefore we must decode their bytes into the data types.
//! When we will decode them, we must care its length and its byte order.
//! See the messages.rs for more detail about each field in the chunk.
//!
//! ## Why will all method return the `Option`?
//!
//! The sent data is **just the bytes**.
//! It won't be necessarily sent the bytes correctly, and the way to check their correctness hasn't been clear yet currently.
//! Therefore the decoder has been defined in such a way as to return the `None` if decoding failed.
//! They are unnecessary to return the `Result` because will never input/output with outer resouces, and also exists the case to need to indicate the end of bytes.
//! e.g. When some decoding termination.
//!
//! [messages.rs]: ./messages.rs.html
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

/// # The byte getter.
///
/// The trait to get some byte from the buffer.
/// This will require that considers the length to get or its byte order.
/// The corresponding of the name to the length/the byte order is following:
///
/// |Word     |Length (in byte)|Byte order    |
/// | :------ | -------------: | :----------- |
/// |\_u8     |1               |Not considered|
/// |\_u16\_be|2               |Big Endian    |
/// |\_u16\_le|2               |Little Endian |
/// |\_u24\_be|3               |Big Endian    |
/// |\_u32\_be|4               |Big Endian    |
/// |\_u32\_le|4               |Little Endian |
/// |\_f64    |8               |IEEE 754      |
///
/// ## Why have "get\_" and "peek\_" existed both?
///
/// The value what must check beforehand will exist.
/// For exmaple, the object end marker of the AMF is 3 bytes, and is consisted of the value of 0x09 (just its marker) to follow after the empty 2 bytes.
/// Note that we can regard their 2 bytes at the head as the string without its marker, but is unnecessary to insert them with the object end marker into the object.
/// Because so, we must regard them as just empty 2 bytes, and must detect the object end marker as one including their 2 bytes.
/// In this case, we will detect them as 0x009 without changing next reading position, then will terminate to decode for the object.
///
/// Therefore the method has been separated its name into "get\_" and "peek\_".
/// This trait expects that you satisfy following things:
///
/// * The method to start with "get\_": decodes bytes then changes next reading position.
/// * The method to start with "peek\_": decodes bytes but don't change next reading position.
pub trait GetByteBuffer {
    /// Takes just 1 byte from the buffer.
    /// If the 1 byte could be read, returns it after containing into the `Option` type, otherwise returns the `None`.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn get_u8(v: &Vec<u8>, offset: &mut usize) -> Option<u8> {
    ///     v.get(*offset).map(
    ///         |byte| {
    ///             *offset +=  1;
    ///             *byte
    ///         }
    ///     )
    /// }
    /// ```
    fn get_u8(&mut self) -> Option<u8>;

    /// Takes 2 bytes as Big Endian from the buffer.
    /// If the 2 bytes could be read, returns them after containing into the `Option` type, otherwise returns the `None`.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn get_u16_be(v: &Vec<u8>, offset: &mut usize) -> Option<u16> {
    ///     v.get(*offset..(*offset + 2)).map(
    ///         |bytes| {
    ///             *offset += 2;
    ///
    ///             let mut array: [u8; 2] = [0; 2];
    ///
    ///             array[0] = bytes[0];
    ///             array[1] = bytes[1];
    ///             u16::from_be_bytes(array)
    ///         }
    ///     )
    /// }
    /// ```
    fn get_u16_be(&mut self) -> Option<u16>;

    /// Takes 2 bytes as Little Endian from the buffer.
    /// If the 2 bytes could be read, returns them after containing into the `Option` type, otherwise returns the `None`.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn get_u16_le(v: &Vec<u8>, offset: &mut usize) -> Option<u16> {
    ///     v.get(*offset..(*offset + 2)).map(
    ///         |bytes| {
    ///             *offset += 2;
    ///
    ///             let mut array: [u8; 2] = [0; 2];
    ///
    ///             array[0] = bytes[0];
    ///             array[1] = bytes[1];
    ///             u16::from_le_bytes(array)
    ///         }
    ///     )
    /// }
    /// ```
    fn get_u16_le(&mut self) -> Option<u16>;

    /// Takes 3 bytes as Big Endian from the buffer.
    /// If the 3 bytes could be read, returns them after containing into the `Option` type, otherwise returns the `None`.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn get_u24_be(v: &Vec<u8>, offset: &mut usize) -> Option<u32> {
    ///     v.get(*offset..(*offset + 3)).map(
    ///         |bytes| {
    ///             *offset += 3;
    ///
    ///             // Note that we must keep the first byte empty because are required to pass as u32.
    ///             let mut array: [u8; 4] = [0; 4];
    ///
    ///             array[1] = bytes[0];
    ///             array[2] = bytes[1];
    ///             array[3] = bytes[2];
    ///             u32::from_be_bytes(array)
    ///         }
    ///     )
    /// }
    /// ```
    fn get_u24_be(&mut self) -> Option<u32>;

    /// Takes 4 bytes as Big Endian from the buffer.
    /// If the 4 bytes could be read, returns them after containing into the `Option` type, otherwise returns the `None`.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn get_u32_be(v: &Vec<u8>, offset: &mut usize) -> Option<u32> {
    ///     v.get(*offset..(*offset + 4)).map(
    ///         |bytes| {
    ///             *offset += 4;
    ///
    ///             let mut array: [u8; 4] = [0; 4];
    ///
    ///             array[0] = bytes[0];
    ///             array[1] = bytes[1];
    ///             array[2] = bytes[2];
    ///             array[3] = bytes[3];
    ///             u32::from_be_bytes(array)
    ///         }
    ///     )
    /// }
    /// ```
    fn get_u32_be(&mut self) -> Option<u32>;

    /// Takes 4 bytes as Little Endian from the buffer.
    /// If the 4 bytes could be read, returns them after containing into the `Option` type, otherwise returns the `None`.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn get_u32_le(v: &Vec<u8>, offset: &mut usize) -> Option<u32> {
    ///     v.get(*offset..(*offset + 4)).map(
    ///         |bytes| {
    ///             *offset += 4;
    ///
    ///             let mut array: [u8; 4] = [0; 4];
    ///
    ///             array[0] = bytes[0];
    ///             array[1] = bytes[1];
    ///             array[2] = bytes[2];
    ///             array[3] = bytes[3];
    ///             u32::from_le_bytes(array)
    ///         }
    ///     )
    /// }
    /// ```
    fn get_u32_le(&mut self) -> Option<u32>;

    /// Takes 8 byte as IEEE 754 from the buffer.
    /// If the 8 bytes could be read, returns them after containing into the `Option` type, otherwise returns the `None`.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn get_f64(v: &Vec<u8>, offset: &mut usize) -> Option<f64> {
    ///     v.get(*offset..(*offset + 8)).map(
    ///         |bytes| {
    ///             *offset += 8;
    ///
    ///             let mut array: [u8; 8] = [0; 8];
    ///
    ///             array[0] = bytes[0];
    ///             array[1] = bytes[1];
    ///             array[2] = bytes[2];
    ///             array[3] = bytes[3];
    ///             array[4] = bytes[4];
    ///             array[5] = bytes[5];
    ///             array[6] = bytes[6];
    ///             array[7] = bytes[7];
    ///             // Note that the f64 type can converted from only the u64 type.
    ///             f64::from_bits(u64::from_be_bytes(array))
    ///         }
    ///     )
    /// }
    /// ```
    fn get_f64(&mut self) -> Option<f64>;

    /// Takes the bytes just as much as what you specified.
    /// If they could be read, returns them after containing into the `Option` type, otherwis returns the `None`.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn get_sliced_bytes(v: &Vec<u8>, offset: &mut usize, len: usize) -> Option<Vec<u8>> {
    ///     v.get(*offset..(*offset + len)).map(
    ///         |bytes| {
    ///             *offset += len;
    ///             bytes.to_vec()
    ///         }
    ///     )
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `len: usize`
    ///
    /// The length to take bytes from the buffer.
    fn get_sliced_bytes(&mut self, len: usize) -> Option<Vec<u8>>;

    /// Takes 1 byte without changing the current position.
    /// If the 1 byte could be read, returns it after containing into the `Option` type, otherwise returns the `None`.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn peek_byte(v: &Vec<u8>, offset: usize) -> Option<u8> {
    ///     // Note that you shouldn't change the current position in this case.
    ///     v.get(offset).map(
    ///         |byte| *byte
    ///     )
    /// }
    /// ```
    fn peek_byte(&self) -> Option<u8>;

    /// Takes the bytes just as much as what you specified without changing the current position.
    /// If they could be read, returns them after containing into the `Option` type, otherwise returns the `None`.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn peek_bytes(v: &Vec<u8>, offset: usize, len: usize) -> Option<Vec<u8>> {
    ///     // Note that you shouldn't change the current position in this case.
    ///     v.get(offset..(offset + len)).map(
    ///         |bytes| {
    ///             bytes.to_vec()
    ///         }
    ///     )
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `len: usize`
    ///
    /// The length to take bytes from the buffer.
    fn peek_bytes(&self, len: usize) -> Option<Vec<u8>>;
}

/// # The decoder for the RTMP chunk
///
/// The trait to decode the bytes into the RTMP chunk.
pub trait RtmpDecoder: GetByteBuffer {
    /// Decodes the chunk basic header.
    /// This expects that you satisfy following points:
    ///
    /// * You regard first 2 bits as the format of the chunk message header.
    /// * If next 6 bits are 0, you regard next 1 byte as the chunk stream id.
    /// * If they are 1, you regard next 2 bytes as the chunk stream id.
    ///   * Note that you must decode them as Little Endian.
    /// * Otherwise you regard its 6 bits as the chunk stream id.
    /// * If you couldn't decode above fields all, return the `None`.
    fn decode_basic_header(&mut self) -> Option<BasicHeader>;

    /// Decodes the chunk message header.
    /// This expects that you satisfy following points:
    ///
    /// * You regard first 3 bytes as the timestamp.
    /// * You regard next 3 bytes as the message length.
    /// * You regard next 1 byte as the message type id.
    /// * You regard next 4 bytes as the message stream id.
    ///   * Note that you must decode them as Little Endian.
    /// * If you couldn't decode above fields all, return the `None`.
    ///
    /// # Parameters
    ///
    /// * `message_format: MessageFormat`
    ///
    /// The chunk message header's format.
    /// You can get this by using `decode_basic_header()` to the buffer.
    /// The returned `MessageHeader` will rely on the value got from above method for its format.
    fn decode_message_header(&mut self, message_format: MessageFormat) -> Option<MessageHeader>;

    /// Decodes the extended timestamp.
    /// This expects that you satisfy following points:
    ///
    /// * If the timestamp in the chunk message header hasn't exceeded 0xffffff, returns the `None`.
    /// * Otherwise you regard the 4 bytes as the extended timestamp.
    /// * You check that the timestamp in the chunk message header has been 0xffffff.
    /// * Though it will be improbable, you set 0xffffff to the timestamp in the chunk message header forcibly if it hasn't been 0xffffff.
    /// * If you couldn't decode the extended timestamp, return the `None`.
    ///
    /// # Parameters
    ///
    /// * `message_header: MessageHeader`
    ///
    /// The message header decoded just before.
    /// It will rely on the timestamp in this value for whether the extended timestamp will exist.
    fn decode_extended_timestamp(&mut self, message_header: MessageHeader) -> Option<Duration>;

    /// Decodes the chunk data as Chunk Size.
    /// This expects that you satisfy following points:
    ///
    /// * You regard the 4 bytes as Chunk Size's value.
    /// * If the most significant bit is 1, you emit the `panic!`.
    /// * If you couldn't decode Chunk Size, return the `None`.
    fn decode_chunk_size(&mut self) -> Option<ChunkData>;

    /// Decodes the chunk data as Bytes Read.
    /// This expects that you satisfy following points:
    ///
    /// * You regard the 4 bytes as Bytes Read's value.
    /// * If you couldn't decode Bytes Read, return the `None`.
    fn decode_bytes_read(&mut self) -> Option<ChunkData>;

    /// Decodes the chunk data as Ping.
    /// This expects that you satisfy following points:
    ///
    /// * You regard first 2 bytes as Ping's event type.  
    /// Ping's event data will be associated with their event types.  
    /// See messages.rs more details about the correspondence of Ping's event type to its event data.
    /// * You regard bytes following after Ping's event type as its event data, and decode their event data corresponding to its event type.
    /// * If you couldn't decode either event type or event data, return the `None`.
    ///
    /// [messages.rs]: ../messages.rs.html
    fn decode_ping(&mut self) -> Option<ChunkData>;

    /// Decodes the chunk data as Server Bandwidth.
    /// This expects that you satisfy following points:
    ///
    /// * You regard the 4 bytes as Server Bandwidth's value.
    /// * If you couldn't decode Server Bandwidth, return the `None`.
    fn decode_server_bandwidth(&mut self) -> Option<ChunkData>;

    /// Decodes the chunk data as Client Bandwidth.
    /// This expects that you satisfy following points:
    ///
    /// * You regard first 4 bytes as Client Bandwidth's value.
    /// * You regard next 1 byte as the limit type of Client Bandwidth.  
    /// See messages.rs more details about its limit type.
    /// * If you couldn't decode either client bandwidth or limit type, return the `None`.
    ///
    /// [messages.rs]: ../messages.rs.html
    fn decode_client_bandwidth(&mut self) -> Option<ChunkData>;

    /// Decodes the chunk data as Audio.
    /// This expects that you satisfy following points:
    ///
    /// * You regard remaining bytes as Audio data.
    /// * If you couldn't decode Audio data, return the `None`.
    ///
    /// Note that you shouldn't consider about the file format and the codec in this moment.
    /// Because the program will become intricately.
    fn decode_audio(&mut self) -> Option<ChunkData>;

    /// Decodes the chunk data as Video.
    /// This expects that you satisfy following points:
    ///
    /// * You regard remaining bytes as Video data.
    /// * If you couldn't decode Video data, return the `None`.
    ///
    /// Note that you shouldn't consider about the file format and the codec in this moment due to the same cause as Audio data.
    fn decode_video(&mut self) -> Option<ChunkData>;

    /// Decodes a part of the chunk data as AMF's number.
    /// This expect that you satisfy following points:
    ///
    /// * You regard the 8 bytes as a floating point number (IEEE 754).
    /// * If you couldn't decode them, return the `None`.
    fn decode_amf_number(&mut self) -> Option<AmfData>;

    /// Decodes a part of the chunk data as AMF's boolean.
    /// This expects that you satisfy following points:
    ///
    /// * You regard the 1 byte as a (AMF's) boolean.
    /// * If it is 0, you regard it as the `false`.
    /// * If it is else, you regard it as the `true`.
    /// * If you couldn't decode it, return the `None`.
    fn decode_amf_boolean(&mut self) -> Option<AmfData>;

    /// Decodes a part of the chunk data as AMF's string.
    /// This expects that you satisfy following points:
    ///
    /// * You regard first 2 bytes as its string's length.
    /// * You take actual string just as much as its length, then decode it as UTF-8.
    /// * If you took the string what its length is 0, may decode it into the `Some("".to_string())`.
    /// * If you couldn't decode above all, return the `None`.
    fn decode_amf_string(&mut self) -> Option<AmfData>;

    /// Decodes a part of the chunk data as AMF's object.
    /// This expects that you satisfy following points:
    ///
    /// * You must associate its name with its value.
    ///   * The name will exist as the string without AMF's marker.
    ///   * The value will exist to follow after its name.  
    ///   This will have AMF's marker.
    /// * If you detected the object end marker following after the empty string key, you must terminate to decode AMF's object.  
    /// Or you may detect it as just 0x000009.
    /// * If you couldn't decode both the name and the value completely, return the `None`.
    fn decode_amf_object(&mut self) -> Option<AmfData>;

    /// Decodes a part of the chunk data as AMF's null.
    /// This expects that you satisfy following points:
    ///
    /// * You regard this as nothing to decode, and return the `AmfData::Null`.
    fn decode_amf_null(&mut self) -> Option<AmfData>;

    /// Decodes a part of the chunk data as AMF's mixed array.
    /// This expects that you satisfy following points:
    ///
    /// * You regard first 4 bytes as the length of this array.
    /// * You regard remaining bytes as what the name and the value will follow after.
    /// * The length will become equal to the count of name/value pairs, though you may discard it because this array will also contain the object end marker.
    /// * If you couldn't decode both the length and the name/value pairs completely, return the `None`.
    fn decode_amf_mixed_array(&mut self) -> Option<AmfData>;

    /// Decodes a part of the chunk data as AMF.
    /// This expects that you satisfy following points:
    ///
    /// * You regard first 1 byte as AMF's data type marker, and regard bytes following it as actual AMF's data.  
    /// See `AmfData` for more details about the correspondence of AMF's data type to actual value.
    /// * If you couldn't decode both the AMF data type and its data completely, return the `None`.
    ///
    /// And keep your mind that you should use the method named as `decode_amf_*` when will decode bytes as AMF.
    /// The correspondence of the AMF data type to the method what should use when decode AMF's data bytes is following:
    ///
    /// |AMF data type (id)|Method                    |
    /// | ---------------: | :----------------------- |
    /// |0                 |`decode_amf_number()`     |
    /// |1                 |`decode_amf_boolean()`    |
    /// |2                 |`decode_amf_string()`     |
    /// |3                 |`decode_amf_object()`     |
    /// |5                 |`decode_amf_null()`       |
    /// |8                 |`decode_amf_mixed_array()`|
    ///
    /// You will this method when will decode actual AMF data after checking id of the AMF data type.
    ///
    /// [`AmfData`]: ../messages/enum.AmfData.html
    fn decode_amf_data(&mut self) -> Option<AmfData>;

    /// Decodes the chunk data as Notify.
    /// This expects that you satisfy following points:
    ///
    /// * You regard the chunk data as the AMF's `MixedArray` what will follow after 2 AMF's `String`s.
    ///   * First string will be "@setDataFrame".
    ///   * Second string will be "onMetaData".
    /// * If you couldn't decode their strings and `MixedArray`, return the `None`.
    fn decode_notify(&mut self) -> Option<ChunkData>;

    /// Decode the chunk data as the response message of the connect command of the NetConnection command.
    /// This expects that you satisfy following points:
    ///
    /// * You decode regarding the response message as what will have following 4 fileds:
    ///   * Command name: "\_result" or "\_error" (`String`)
    ///   * Transaction id: the same value as what the request of connect has (`Number`)
    ///   * Properties: an AMF's `Object`
    ///   * Information: See the `InfoObject` (`Object`)
    /// * If you couldn't decode above fields all, return the `None`.
    ///
    /// [`CommandObject`]: ../messages/struct.CommandObject.html
    /// [`InfoObject`]: ../messages/struct.InfoObject.html
    fn decode_invoke_connect_result(&mut self) -> Option<ChunkData>;

    /// Decodes the chunk data as the response message of the releaseStream command of the NetConnection command.
    /// This expects that you satisfy following points:
    ///
    /// * You decode regarding the response message as what will have following 4 fields:
    ///   * Command name: "\_result" or "\_error" (`String`)
    ///   * Transaction id: the same value as what the releaseStream request has (`Number`)
    ///   * Command object: probable nothing (`Null`)
    /// * If you couldn't decode above fields all, return the `None`.
    fn decode_invoke_release_stream_result(&mut self) -> Option<ChunkData>;

    /// Decodes the chunk data as the response message of the createStream command of the NetConnection command.
    /// This expects that you satisfy following points:
    ///
    /// * You decode regarding the response message as what will have following 4 fields:
    ///   * Command name: "\_result" or "\_error" (`String`)
    ///   * Transaction id: the same value as what the createStream request has (`Number`)
    ///   * Command object: probable nothing (`Null`)
    ///   * Message stream id: one what the server is holding for this client (`Number`)
    /// * If you couldn't decode above field all, return the `None`.
    fn decode_invoke_create_stream_result(&mut self) -> Option<ChunkData>;

    /// Decodes the response message of the FCPublish command.
    /// This expects that you satisfy following points:
    ///
    /// * You decode regarding the response message as what will have following 1 field:
    ///   * Command name: "onFCPublish" (`String`)
    /// * If you couldn't above field, return the `None`.
    fn decode_invoke_on_fc_publish(&mut self) -> Option<ChunkData>;

    /// Decodes the response message of the NetStream command.
    /// This expects that you satisfy following points:
    ///
    /// * You deocode regarding the response message as what will have following 4 fields:
    ///   * Command name: "onStatus" (`String`)
    ///   * Transaction id: 0 (`Number`)
    ///   * Command object: probable nothing (`Null`)
    ///   * Info object: See the `InfoObject` (`Object`)
    /// * If you couldn't decode above field all, return the `None`.
    ///
    /// [`InfoObject`]: ../messages/struct.InfoObject.html
    fn decode_invoke_on_status(&mut self) -> Option<ChunkData>;

    /// Decodes the invocation request message.
    /// This expects that you satisfy following points:
    ///
    /// * You decode regarding the invocation request as what will have following messages:
    ///   * "connect"
    ///   * "releaseStream"
    ///   * "FCPublish"
    ///   * "createStream"
    ///   * "publish"
    ///
    /// And you decode regarding above messages as what will have following fields respectively:
    ///
    /// ## connect
    ///
    /// * Command name: "connect" (`String`)
    /// * Transaction id: 1 (`Number`)
    /// * Command object: See the `CommandObject` (`Object`)
    /// * Optional argument: In the official specification paper, this has been specified the `Object`, though we can't check the existence of this field.  
    /// Therefore you won't be required to consider about this field.
    ///
    /// ## releaseStream
    ///
    /// * Command name: "releaseStream" (`String`)
    /// * Transaction id: probable 2 (`Number`)
    /// * Command object: probable nothing (`Null`)
    /// * Playpath: playpath in "rtmp://example.com/appName/*playpath*" (if it is specified) (`String`)
    ///
    /// ## FCPublish
    ///
    /// * Command name: "FCPublish" (`String`)
    /// * Transaction id: probable 3 (`Number`)
    /// * Command object: probable nothing (`Null`)
    /// * Playpath: playpath in "rtmp://example.com/appName/*playpath*" (if it is specified) (`String`)
    ///
    /// ## createStream
    ///
    /// * Command name: "createStream" (`String`)
    /// * Transaction id: probable 4 (`Number`)
    /// * Command object: probable nothing (`Null`)
    ///
    /// ## publish
    ///
    /// * Command name: "publish" (`String`)
    /// * Transaction id: probable 5 (`Number`)
    /// * Command object: probable nothing (`Null`)
    /// * Playpath: playpath in "rtmp://example.com/appName/*playpath*" (if it is specified) (`String`)
    /// * Playtype: either "live", "record" or "append" (`String`)
    ///
    /// * If you couldn't decode above messages, return the `None`
    ///
    /// [`CommandObject`]: ../messages/sturct.CommandObject.html
    fn decode_invoke(&mut self) -> Option<ChunkData>;

    /// Decodes the chunk data as Unknown bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You regard remaining bytes as some unknown bytes.  
    /// You aren't required to consider that their bytes is what in this moment.
    /// * You may pass them through other processes keeping them as be.
    fn decode_unknown(&mut self) -> Option<ChunkData>;

    /// Decodes the chunk data.
    /// This expects that you satisfy following points:
    ///
    /// * You must associate the message type id with actual chunk data.  
    /// See the `MessageType` and the `ChunkData` for more details about their correspondence.
    /// * If you couldn't decode the chunk data, return the `None`.
    ///
    /// And keep your mind that you should use the method named as "decode_*" when will decode the chunk data.
    /// The correspondence of the message type and the method what should use when decode the chunk data is following:
    ///
    /// |Message type id  |Method                     |
    /// | --------------: | :------------------------ |
    /// |1                |`decode_chunk_size()`      |
    /// |3                |`decode_bytes_read()`      |
    /// |4                |`decode_ping()`            |
    /// |5                |`decode_server_bandwidth()`|
    /// |6                |`decode_client_bandwidth()`|
    /// |8                |`decode_audio()`           |
    /// |9                |`decode_video()`           |
    /// |18               |`decode_notify()`          |
    /// |20               |`decode_invoke()`          |
    /// |other            |`decode_unknown()`         |
    ///
    /// You will use this method when will decode actual chunk data after checking the message type id in the chunk message header.
    ///
    /// # Parameters
    ///
    /// * `message_type: MessageType`
    ///
    /// The message type input in the chunk message header.
    /// The chunk data must be associated actual content with the message type ida in the chunk message header.
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
                    println!("The most significant bit is 1!");
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

    fn decode_audio(&mut self) -> Option<ChunkData> {
        let offset = self.offset();
        let len = self.len();

        self.get_sliced_bytes(len - offset).map(
            |bytes| ChunkData::Audio(bytes)
        )
    }

    fn decode_video(&mut self) -> Option<ChunkData> {
        let offset = self.offset();
        let len = self.len();

        self.get_sliced_bytes(len - offset).map(
            |bytes| ChunkData::Video(bytes)
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
                self.get_sliced_bytes(3); // Skips the AMF's object end marker.
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

    fn decode_amf_mixed_array(&mut self) -> Option<AmfData> {
        let mut mixed_array: HashMap<String, AmfData> = HashMap::new();

        self.get_u32_be(); // The mixed array can skip the length field because has an object end marker at the end of its array.

        while let Some(bytes) = self.peek_bytes(3) {
            if bytes == &AmfData::OBJECT_END_SEQUENCE {
                self.get_sliced_bytes(3); // Skips the AMF's object end marker.
                break;
            }

            let key = self.decode_amf_string().unwrap().string().unwrap();
            let value = self.decode_amf_data().unwrap();

            mixed_array.insert(key, value);
        }

        Some(AmfData::MixedArray(mixed_array))
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
                    MixedArray => self.decode_amf_mixed_array(),
                    _ => None
                }
            }
        )
    }

    fn decode_notify(&mut self) -> Option<ChunkData> {
        self.decode_amf_data().and_then(
            |s| s.string().and_then(
                |command| {
                    use crate::messages::{
                        ChunkData::Notify,
                        NotifyCommand::*
                    };

                    if command == "@setDataFrame" {
                        let data_frame = self.decode_amf_data().and_then(
                            |s| s.string()
                        ).unwrap();
                        let meta_data = self.decode_amf_data().and_then(
                            |object| object.object().map(
                                |properties| {
                                    let meta_data: MetaData = properties.into();

                                    meta_data
                                }
                            )
                        ).unwrap();

                        Some(
                            Notify(
                                SetDataFrame {
                                    data_frame,
                                    meta_data
                                }
                            )
                        )
                    } else {
                        println!("Unknown notify command: {}", command);

                        let len = self.len();
                        let offset = self.offset();

                        self.get_sliced_bytes(len - offset).map(
                            |bytes| Notify(Unknown(bytes))
                        )
                    }
                }
            )
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
                                println!("Unknown invoke command: {}", command);

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
            Audio => self.decode_audio(),
            Video => self.decode_video(),
            Notify => self.decode_notify(),
            Invoke => self.decode_invoke(),
            _ => self.decode_unknown()
        }
    }
}
