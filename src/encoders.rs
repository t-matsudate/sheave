//! # The traits to encode the chunk messages.
//!
//! First, we will send the chunk as bytes (that is, `Vec<u8>`).
//! Therefore we must encode it into bytes.
//! When we will encode them, we must care its format and its byte order.
//! See messages.rs for more detail about each field in the chunk.
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

/// # The byte putter
///
/// The trait to put some value into the buffer.
/// This will require that consider the length to put or its byte order.
/// The correspondence of the name to the length/the byte order is following:
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
pub trait PutByteBuffer {
    /// Puts just 1 byte into the buffer.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn put_u8(v: &mut Vec<u8>, byte: u8) {
    ///     v.push(byte);
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `byte: u8`
    ///
    /// A 1 byte value to put into the buffer.
    fn put_u8(&mut self, byte: u8);

    /// Puts 2 bytes as Big Endian into the buffer.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn put_u16_be(v: &mut Vec<u8>, byte: u16) {
    ///     v.extend_from_slice(&byte.to_be_bytes());
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `byte: u16`
    ///
    /// A 2 bytes value to put into the buffer.
    /// This is expected to convert into the `[u8; 2]` type as Big Endian.
    fn put_u16_be(&mut self, byte: u16);

    /// Puts 2 bytes as Little Endian into the buffer.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn put_u16_le(v: &mut Vec<u8>, byte: u16) {
    ///     v.extend_from_slice(&byte.to_le_bytes());
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `byte: u16`
    ///
    /// A 2 bytes value to put into the buffer.
    /// This is expected to convert into the `[u8; 2]` type as Little Endian.
    fn put_u16_le(&mut self, byte: u16);

    /// Puts 3 bytes as Big Endian into the buffer.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn put_u24_be(v: &mut Vec<u8>, byte: u32) {
    ///     // The first byte won't be needed because this method is requiring just 3 bytes.
    ///     v.extend_from_slice(&byte.to_be_bytes()[1..]);
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `byte: u32`
    ///
    /// A 4 bytes value to put into the buffer.
    /// This is expected to convert into the `[u8; 4]` type as Big Endian.
    /// Though this is 4 bytes because the type for 3 bytes isn't existing yet, keep this value up to 0x00ffffff.
    fn put_u24_be(&mut self, byte: u32);

    /// Puts 4 bytes as Big Endian into the buffer.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn put_u32_be(v: &mut Vec<u8>, byte: u32) {
    ///     v.extend_from_slice(&byte.to_be_bytes());
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `byte: u32`
    ///
    /// A 4 bytes value to put into the buffer.
    /// This is expected to convert into the `[u8; 4]` type as Big Endian.
    fn put_u32_be(&mut self, byte: u32);

    /// Puts 4 bytes as Little Endian into the buffer.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn put_u32_le(v: &mut Vec<u8>, byte: u32) {
    ///     v.extend_from_slice(&byte.to_le_bytes());
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `byte: u32`
    ///
    /// A 4 bytes value to put into the buffer.
    /// This is expected to convert into the `[u8; 4]` type as Little Endian.
    fn put_u32_le(&mut self, byte: u32);

    /// Puts 8 bytes as IEEE 754 into the buffer.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// fn put_f64(v: &mut Vec<u8>, byte: f64) {
    ///     // f64::to_bits() makes the u64 value from the f64 value.
    ///     v.extend_from_slice(&byte.to_bits().to_be_bytes());
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `byte: f64`
    ///
    /// A 8 bytes value to put into the buffer.
    /// This is expected to convert into the `[u8; 8]` type as Big Endian.
    fn put_f64(&mut self, byte: f64);

    /// Puts the bytes into the buffer as it be.
    /// This expects that you write the code equivalent to following:
    ///
    /// ```
    /// // Note that "mut" is a pattern, and can't use in the trait defenition.
    /// fn put_bytes(v: &mut Vec<u8>, mut bytes: Vec<u8>) {
    ///     v.append(&mut bytes);
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `bytes: Vec<u8>`
    ///
    /// The bytes to put into the buffer.
    fn put_bytes(&mut self, bytes: Vec<u8>);
}

/// # The encoder for the RTMP chunk
///
/// The trait to encode the RTMP chunk into the bytes.
pub trait RtmpEncoder: PutByteBuffer {
    /// Encodes the chunk basic header into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this header into the buffer at head.
    ///
    /// See messages.rs for more detail about the chunk basic header.
    ///
    /// # Parameters
    ///
    /// * `basic_header: BasicHeader`
    ///
    /// The chunk basic header to put into the buffer.
    /// See `BasicHeader` for more detail about this type.
    ///
    /// [messages.rs]: ../messages.rs.html
    /// [`BasicHeader`]: ../messages/struct.BasicHeader.html
    fn encode_basic_header(&mut self, basic_header: BasicHeader);

    /// Encodes the chunk message header into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this header into just after the chunk basic header in the buffer.
    ///
    /// If this header is Format 3, you must input nothing.
    /// See messages.rs for more detail about the chunk basic header.
    ///
    /// # Parameters
    ///
    /// * `message_header: MessageHeader`
    ///
    /// The chunk message header to put into the buffer.
    /// See `MessageHeader` for more detail about this type.
    ///
    /// [messages.rs]: ../messages.rs.html
    /// [`MessageHeader`]: ../messages/enum.MessageHeader.html
    fn encode_message_header(&mut self, message_header: MessageHeader);

    /// Encodes the extended timestamp into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You this into just after the chunk message header in the buffer.
    ///   * If this is `None`, you encode nothing.
    ///   * Otherwise you put this into the buffer as **4 bytes** value.
    ///
    /// See messages.rs for more detail about the extended timestamp.
    ///
    /// # Parameters
    ///
    /// * `timestamp: Option<Duration>`
    ///
    /// The extended timestamp to put into the buffer.
    /// Consider how to process the value above 0xffffffff.
    ///
    /// [messages.rs]: ../messages.rs.html
    fn encode_extended_timestamp(&mut self, timestamp: Option<Duration>);

    /// Encodes the chunk size value into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into following place:
    ///   * If the extended timestamp is existing, you put this into just after the extended timestamp in the buffer.
    ///   * Otherwise you put this into just after the chunk message header in the buffer.
    ///
    /// See messages.rs for more detail about the chunk size.
    ///
    /// # Parameters
    ///
    /// * `chunk_size: u32`
    ///
    /// The chunk size to put into the buffer.
    /// Consider how to process the value above 0x7fffffff.
    ///
    /// [messages.rs]: ../messages.rs.html
    fn encode_chunk_size(&mut self, chunk_size: u32);

    /// Decodes the size of total read bytes into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into following place:
    ///   * If the extended timestamp is existing, you put this into just after the extended timestamp in the buffer.
    ///   * Otherwise you put this into just after the chunk message header in the buffer.
    ///
    /// See messages.rs for more detail about the size of read bytes.
    ///
    /// # Parameter
    ///
    /// * `bytes_read: u32`
    ///
    /// The size of read bytes to put into the buffer.
    ///
    /// [messages.rs]: ../messages.rs.html
    fn encode_bytes_read(&mut self, bytes_read: u32);

    /// Encodes the ping event into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into following place:
    ///   * If the extended timestamp is existing, you put this into just after the extended timestamp in the buffer.
    ///   * Otherwise you put this into just after the chunk message header in the buffer.
    /// * You put Ping's data following after its event type (2 bytes).
    ///
    /// See messages.rs for more detail about Ping.
    ///
    /// # Parameters
    ///
    /// * `ping_data: PingData`
    ///
    /// Ping's data to put into the buffer.
    /// See `PingType` and `PingData` for more detail about this type.
    ///
    /// [messages.rs]: ../messages.rs.html
    /// [`PingType`]: ../messages/enum.PingType.html
    /// [`PingData`]: ../messages/enum.PingData.html
    fn encode_ping(&mut self, ping_data: PingData);

    /// Encodes server side bandwidth into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into following place:
    ///   * If the extended timestamp is existing, you put this into just after the extended timestamp in the buffer.
    ///   * Otherwise you put this into just after the chunk message header in the buffer.
    ///
    /// See messages.rs for more detail about the server bandwidth.
    ///
    /// # Parameters
    ///
    /// * `bandwidth: u32`
    ///
    /// The server side bandwidth to put into the buffer.
    ///
    /// [messages.rs]: ../messages.rs.html
    fn encode_server_bandwidth(&mut self, bandwidth: u32);

    /// Encodes the client side bandwidth and its limit type into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put these into following places:
    ///   * If the extended timestamp is existing, you put these into just after the extended timestamp in the buffer.
    ///   * Otherwise you put these into just after the chunk message header in the buffer.
    /// * You put the limit type (1 byte) after the client side bandwidth.  
    ///
    /// See messages.rs for more detail about the client side bandwidth.
    ///
    /// # Parameters
    ///
    /// * `bandwidth: u32`
    ///
    /// The client side bandwidh to put into the buffer.
    ///
    /// * `limit_type: LimitType`
    ///
    /// The limit type for the client side bandwidth to put into the buffer.
    /// See `LimitType` for more detail about this type.
    ///
    /// [messages.rs]: ../messages.rs.html
    /// [`LimitType`]: ../messages/enum.LimitType.html
    fn encode_client_bandwidth(&mut self, bandwidth: u32, limit_type: LimitType);

    /// Encodes the audio data into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into following place:
    ///   * If the extended timestamp is existing, you put this into just after the extended timestamp in the buffer.
    ///   * Otherwise you put this into just after the chunk message header in the buffer.
    ///
    /// Note that you shouldn't consider about the file format and the codec in this moment.
    /// Because the program will become intricately.
    /// See flv.rs for more detail about the audio data.
    ///
    /// # Parameter
    ///
    /// * `bytes: Vec<u8>`
    ///
    /// The audio data to put into this buffer.
    ///
    /// [flv.rs] ../flv.rs.html
    fn encode_audio(&mut self, bytes: Vec<u8>);

    /// Encodes the video data into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into following place:
    ///   * If the extended timestamp is existing, you put this into just after the extended timestamp in the buffer.
    ///   * Otherwise you put this into just after the chunk message header in the buffer.
    ///
    /// Note that you shouldn't consider about the file format and the codec in this moment.
    /// Because the program will become intricately.
    /// See flv.rs for more detail about the video data.
    ///
    /// # Parameters
    ///
    /// * `bytes: Vec<u8>`
    ///
    /// The video data to put into the buffer.
    ///
    /// [flv.rs]: ../flv.rs.html
    fn encode_video(&mut self, bytes: Vec<u8>);

    /// Encodes f64 value into the bytes of the AMF's `Number`.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into the buffer following after an AMF data type marker of the number(0x00).
    ///
    /// # Parameters
    ///
    /// * `number: f64`
    ///
    /// The number to put into the buffer.
    fn encode_amf_number(&mut self, number: f64);

    /// Encodes bool value into the bytes of AMF's `Boolean`.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into the buffer following after an AMF data type marker of the boolean (0x01).
    /// * You regard false as 0, true as otherwise.
    ///
    /// # Parameters
    ///
    /// * `boolean: bool`
    ///
    /// The boolean value to put into the buffer.
    fn encode_amf_boolean(&mut self, boolean: bool);

    /// Encodes string value into the bytes of the AMF's `String`.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into the buffer following after an AMF data type marker of the string (0x02).
    /// * At this moment, you must put the length of this string (2 bytes **integer**, Big Endian) before actual string.  
    /// The length must be equal to actual string length.
    /// * You must encode this sure as UTF-8.
    ///
    /// # Parameters
    ///
    /// * `string: String`
    ///
    /// The string to put into this buffer.
    /// You make sure that this is encoded as UTF-8.
    fn encode_amf_string(&mut self, string: String);

    /// Encodes hash map value into the bytes of AMF's `Object`.
    /// This expects that you satisfy following pints:
    ///
    /// * You put this into the buffer following after the AMF data type marker of the object (0x03).
    /// * At this moment, you make sure that the object has consisted of name/value pairs:
    ///   * The name is a string without only the AMF data type marker.
    ///   * The value is some AMF's value.
    ///   * AMF's object must terminate by AMF's object end marker (0x09), and the object end marker must follow after a key of empty string.  
    ///   That is, the object end marker will consist of 0x000009 (3 bytes).
    /// * You must encode the name as as UTF-8.
    ///
    /// # Parameters
    ///
    /// * `object: HashMap<String, AmfData>`
    ///
    /// The object to put into the buffer.
    /// You make sure that the key is encoded as UTF-8.
    fn encode_amf_object(&mut self, object: HashMap<String, AmfData>);

    /// Encodes the AMF's `Null` into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You regard this as what has no value.  
    /// That is, you put only the AMF data type marker into the buffer.
    fn encode_amf_null(&mut self);

    /// Encodes hash map value into the bytes of AMF's `MixedArray`.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into the buffer following after the AMF data type marker of mixed array (0x08).
    /// * At this moment, you make sure that the mixed array has consisted of name/value pairs with their length.
    ///   * The length is a 4 bytes **integer** (Big Endian).  
    ///   The length must be equal to actual count of name/value pairs, though we can ignore this value because the mixed array has the object end marker.
    ///   * The name is a string without only the AMF data type marker.
    ///   * The value is some AMF's value.
    ///   * AMF's mixed array must terminate by AMF's object end marker (0x09), and the object end marker must follow after a key of empty string.  
    ///   That is, the object end marker will consist of 0x000009 (3 bytes).
    ///   * You must encode the name as UTF-8.
    ///
    /// # Parameters
    ///
    /// * `mixed_array: HashMap<String, AmfData>`
    ///
    /// The mixed array to put into the buffer.
    /// You make sure that the key is encoded as UTF-8.
    fn encode_amf_mixed_array(&mut self, mixed_array: HashMap<String, AmfData>);

    /// Encodes the AMF's value into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into the buffer following after its data type marker.  
    /// At this moment, you should use the method named as "encode\_amf\_\*" corresponded to its data type marker.  
    /// The correspondence of the AMF data type to the method what should use when encode AMF's data bytes is following:
    ///
    /// |AMF data type|Method                    |
    /// | :---------- | :----------------------- |
    /// |`Number`     |`encode_amf_number()`     |
    /// |`Boolean`    |`encode_amf_boolean()`    |
    /// |`String`     |`encode_amf_string()`     |
    /// |`Object`     |`encode_amf_object()`     |
    /// |`Null`       |`encode_amf_null()`       |
    /// |`MixedArray` |`encode_amf_mixed_array()`|
    ///
    /// See `AmfDataType` for more detail about the AMF data type marker.
    ///
    /// # Parameters
    ///
    /// * `data: AmfData`
    ///
    /// Some AMF's value to put into the buffer.
    /// See `AmfData` for more detail about the AMF's value.
    ///
    /// [`AmfDataType`]: ../messages/enum.AmfDataType.html
    /// [`AmfData`]: ../messages/enum.AmfData.html
    fn encode_amf_data(&mut self, data: AmfData);

    /// Encodes the notify message into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into following place:
    ///   * If the extended timestamp is existing, you put this into just after the extended timestamp in the buffer.
    ///   * Otherwise you put this into just after the chunk message header in the buffer.
    /// * At this morment, you put following fields into the buffer respectively:
    ///   * First, "@setDataFrame" (`String`).
    ///   * Second, "onMetaData" (`String`).
    ///   * Then, actual MetaData for FLV (`MixedArray`).
    ///
    /// See `AmfDataType` and `MetaData` for more detail about the AMF data type marker and the content of actual metadata.
    ///
    /// # Parameters
    ///
    /// * `notify_command: NotifyCommand`
    ///
    /// The content of notify message to put into this buffer.
    /// See `AmfData` and `NotifyCommand` for more detail about this type.
    ///
    /// [`AmfDataType`]: ../messages/enum.AmfDataType.html
    /// [`AmfData`]: ../messages/enum.AmfData.html
    /// [`MetaData`]: ../flv/struct.MetaData.html
    /// [`NotifyCommand`]: ../messages/enum.NotifyCommand.html
    fn encode_notify(&mut self, notify_command: NotifyCommand);

    /// Encodes the NetConnection command of the invocation message into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You encode regarding the NetConnection command as what will have following messages:
    ///   * "connect"
    ///   * "releaseStream"
    ///   * "createStream"
    ///
    /// And you encode regarding above messages as what will have following fields respectively:
    ///
    /// ## connect
    ///
    /// In request:
    ///
    /// * Command name: "connect" (`String`).
    /// * Transaction id: 1 (`Number`).
    /// * Command object: See the `CommandObject` (`Object`)
    /// * Optional Argument: In the official specification paper, this has been specified the `Object`, though we can't check the existence of this field.  
    /// Therefore you won't be required to consider about this field.
    ///
    /// In response:
    ///
    /// * Command name: "\_result" or "\_error" (`String`)
    /// * Transaction id: the same value as what the request of the connect has (`Number`)
    /// * Properties: an AMF's `Object`
    /// * Information: See the `InfoObject` (`Object`).
    ///
    /// ## releaseStream
    ///
    /// In request:
    ///
    /// * Command name: "releaseStream" (`String`)
    /// * Transaction id: probable 2 (`Number`)
    /// * Command object: See the `CommandObject` (`Object`).  
    /// Note that you must put an AMF's `Null` into the buffer instead if this is nothing.
    /// * Playpath: playpath in "rtmp://example.com/appName/*playpath*" (if is specified) (`String`)
    ///
    /// In response:
    ///
    /// * Command name: "\_result" or "\_error" (`String`)
    /// * Transaction id: the same value as what the request of releaseStream has (`Number`)
    /// * Command object: See the `CommandObject` (`Object`).  
    /// Note that you must put AMF's `Null` into the buffer instead if this is nothing.
    ///
    /// ## createStream
    ///
    /// In request:
    ///
    /// * Command name: "createStream" (`String`)
    /// * Transaction id: probable 4 (`Number`)
    /// * Command object: see the `CommandObject` (`Object`).  
    /// Note that you must put AMF's `Null` into the buffer instead if this is nothing.
    ///
    /// In response:
    ///
    /// * Command name: "\_result" or "\_error" (`String`)
    /// * Transaction id: the same value as what the request of createStream has (`Number`)
    /// * Command object: See the `CommandObject` (`Object`).  
    /// Note that you must put AMF's `Null` into the buffer instead if this is nothing.
    /// * Message stream id: new stream message id for its client (`Number`)
    ///
    /// # Parameters
    ///
    /// * `net_connection: NetConnectionCommand`
    ///
    /// The NetConnection command of invocation message to put into the buffer.
    /// See the `NetConnectionCommand` for more detail about this type.
    ///
    /// [`CommandObject`]: ../messages/struct.CommandObject.html
    /// [`InfoObject`]: ../messages/struct.InfoObject.html
    /// [`NetConnectionCommand`]: ../messages/enum.NetConnectionCommand.html
    fn encode_invoke_net_connection(&mut self, net_connection: NetConnectionCommand);

    /// Encodes the NetStream command of the invocation message into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You encode regarding the NetStream command as what will have following messages:
    ///   * "publish"
    ///
    /// And you encode regarding above messages as what will have following fields respectively:
    ///
    /// ## publish
    ///
    /// In request:
    ///
    /// * Command name: "publish" (`String`)
    /// * Transaction id: probable 5 (`Number`)
    /// * Command object: an AMF's `Null`
    /// * Playpath: playpath in "rtmp://example.com/appName/*playpath*" (if is specified) (`String`)
    /// * Playtype: either "live", "record" or "append" (`String`)
    ///
    /// In response:
    ///
    /// * Command name: "onStatus" (`String`)
    /// * Transaction id: 0 (`Number`)
    /// * Command object: an AMF's `Null`
    /// * Info object: See the `InfoObject` (`Object`)
    ///
    /// # Parameters
    ///
    /// * `net_stream: NetStreamCommand`
    ///
    /// The NetStream command to put into the buffer.
    /// See the `NetStreamCommand` for more detail about this type.
    ///
    /// [`InfoObject`]: ../messages/struct.InfoObject.html
    /// [`NetStreamCommand`]: ../messages/enum.NetStreamCommand.html
    fn encode_invoke_net_stream(&mut self, net_stream: NetStreamCommand);

    /// Encodes the FCPublish command of the invocation message into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You encode regarding the FCPublish command as what will have following messages:
    ///   * "FCPublish"
    ///
    /// And you encode regarding above messages as what will have following fields respectively:
    ///
    /// ## FCPublish
    ///
    /// In request:
    ///
    /// * Command name: "FCPublish" (`String`)
    /// * Transaction id: probable 3 (`Number`)
    /// * Command object: an AMF's `Null`
    /// * Playpath: playpath in "rtmp://example.com/appName/*playpath*" (if is specified) (`String`)
    ///
    /// In response:
    ///
    /// * Command name: "onFCPublish" (`String`)
    ///
    /// # Parameters
    ///
    /// * `fc_publish: FcPublishCommand`
    ///
    /// The FCPublish command to put into the buffer.
    /// See the `FcPublishCommand` for more detail about this type.
    ///
    /// [`FcPublishCommand`]: ../messages/enum.FcPublishCommand.html
    fn encode_invoke_fc_publish(&mut self, fc_publish: FcPublishCommand);

    /// Encodes some invocation message into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * You put this into following place:
    ///   * If the extended timestamp is existing, you put this into just after the extended timestamp in the buffer.
    ///   * Otherwise you put this into just after the chunk message header in the buffer.
    /// * At this moment, you put following fields into the buffer respectively:
    ///   * Some command name of subcommands (`String`)
    ///   * Transaction id (`Number`)
    ///   * Actual data for subcommands (some AMF's value)
    /// * You should use the method named as "encode\_invoke\_\*" corresponded to the type of the command.  
    /// The correspondence of the the command to the method what should use when encode the command is following:
    ///
    /// |Command        |Method                       |
    /// | :------------ | :-------------------------- |
    /// |`NetConnection`|`encode_invoke_net_connect()`|
    /// |`NetStream`    |`encode_invoke_net_stream()` |
    /// |`FCPublish`    |`encode_invoke_fc_publish()` |
    ///
    /// # Parameters
    ///
    /// * `invoke: InvokeCommand`
    ///
    /// The command of invocation message to put into the buffer.
    /// See the `InvokeCommand` for more detail about this type.
    ///
    /// [`InvokeCommand`]: ../messages/enum.InvokeCommand.html
    fn encode_invoke(&mut self, invoke: InvokeCommand);

    /// Puts some unknown bytes into the buffer as it be.
    /// This expects that you satisfy following points:
    ///
    /// * If the extended timestamp is existing, you put this into just after the extended timestamp in the buffer.
    /// * Otherwise you put this into just after the chunk message header in the buffer.
    ///
    /// # Parameters
    ///
    /// * `unknown: Vec<u8>`
    ///
    /// Some unknown bytes to put into the buffer.
    fn encode_unknown(&mut self, unknown: Vec<u8>);

    /// Encodes some chunk data into the bytes.
    /// This expects that you satisfy following points:
    ///
    /// * If the extended timestamp is existing, you put this into just after the extended timestamp in the buffer.
    /// * Otherwise you put this into just after the chunk message header in the buffer.
    /// * If the chunk data is nothing, you put nothing into the buffer.
    /// * And you should use the method named "encode\_\*" corresponded to the type of the chunk pattern.  
    /// The correspondence of the chunk pattern to the method what should use when encode the chunk data is following:
    ///
    /// |ChunkData        |Method                     |
    /// | :-------------- | :------------------------ |
    /// |`ChunkSize`      |`encode_chunk_size()`      |
    /// |`BytesRead`      |`encode_bytes_read()`      |
    /// |`PingData`       |`encode_ping()`            |
    /// |`ServerBandwidth`|`encode_server_bandwidth()`|
    /// |`ClientBandwidth`|`encode_client_bandwidth()`|
    /// |`Audio`          |`encode_audio()`           |
    /// |`Video`          |`encode_video()`           |
    /// |`Notify`         |`encode_notify()`          |
    /// |`Invoke`         |`encode_invoke()`          |
    /// |Otherwise        |`encode_unknown()`         |
    ///
    /// # Parameters
    ///
    /// * `chunk_data: Option<ChunkData>`
    ///
    /// The chunk data to put into the buffer.
    /// If chunk data to encode is nothing, pass the `None`.
    /// See the `ChunkData` for more detail about this type.
    ///
    /// [`ChunkData`]: ../messages/enum.ChunkData.html
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

    fn encode_audio(&mut self, bytes: Vec<u8>) {
        self.put_bytes(bytes);
    }

    fn encode_video(&mut self, bytes: Vec<u8>) {
        self.put_bytes(bytes);
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

    fn encode_amf_mixed_array(&mut self, mixed_array: HashMap<String, AmfData>) {
        self.put_u8(AmfDataType::MixedArray as u8);
        self.put_u32_be(mixed_array.len() as u32);

        for (key, value) in mixed_array {
            self.put_u16_be(key.len() as u16);
            self.put_bytes(key.into_bytes());
            self.encode_amf_data(value);
        }

        self.put_bytes(AmfData::OBJECT_END_SEQUENCE.to_vec());
    }

    fn encode_amf_data(&mut self, data: AmfData) {
        use crate::messages::AmfData::*;

        match data {
            Number(number) => self.encode_amf_number(number),
            Boolean(boolean) => self.encode_amf_boolean(boolean),
            String(string) => self.encode_amf_string(string),
            Object(object) => self.encode_amf_object(object),
            Null => self.encode_amf_null(),
            MixedArray(mixed_array) => self.encode_amf_mixed_array(mixed_array),
            _ => ()
        }
    }

    fn encode_notify(&mut self, notify_command: NotifyCommand) {
        use crate::messages::NotifyCommand::*;

        match notify_command {
            SetDataFrame {
                data_frame,
                meta_data
            } => {
                self.encode_amf_string("@setDataFrame".to_string());
                self.encode_amf_string(data_frame);
                self.encode_amf_object(meta_data.into());
            },
            Unknown(bytes) => self.put_bytes(bytes)
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
                self.encode_amf_object(information.into());
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

    fn encode_invoke_net_stream(&mut self, net_stream: NetStreamCommand) {
        use crate::messages::NetStreamCommand::*;

        match net_stream {
            Publish {
                transaction_id,
                play_path,
                play_type
            } => {
                self.encode_amf_string("publish".to_string());
                self.encode_amf_number(transaction_id as f64);
                self.encode_amf_null();
                self.encode_amf_string(play_path);
                self.encode_amf_string(play_type.into());
            },
            OnStatus {
                transaction_id,
                info_object
            } => {
                self.encode_amf_string("onStatus".to_string());
                self.encode_amf_number(transaction_id as f64);
                self.encode_amf_null();
                self.encode_amf_object(info_object.into());
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
            NetStream(net_stream) => self.encode_invoke_net_stream(net_stream),
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
                    Audio(bytes) => self.encode_audio(bytes),
                    Video(bytes) => self.encode_video(bytes),
                    Notify(notify_command) => self.encode_notify(notify_command),
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
