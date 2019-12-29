//! # The types for FLV file format.
//!
//! We will be required to store sent Audio/Video data from the client because they must send the to other clients too.
//! Therefore we must store several segments of Audio/Video data sent from the client just as much as needed.
//! The FLV file format consists of following fields respectively:
//!
//! 1. The FLV header
//! 2. The FLV bodies
//!    * The sequence of following fields:
//!      * The size of previous tag (unsigned 4 bytes number but the size of first tag must be 0)
//!      * The FLV tag
//!
//! ## The FLV header
//!
//! The FLV header consists of following fields respectively:
//!
//! 1. Signatures (array of unsigned 1 byte, total of 3 bytes)
//! 2. Version (unsigned 1 byte number)
//! 3. Reserved (unsigned 5 **bits**)
//! 4. Audio flag (unsigned 1 **bit**)
//! 5. Reserved (unsigned 1 **bit**)
//! 6. Video flag (unsigned 1 **bit**)
//! 7. Data offset (unsigned 4 bytes number, in byte)
//!
//! ### Signatures
//!
//! This is "FLV" as the string.
//!
//! ### Version
//!
//! The version for this file.
//! For example, if this is 1, it indicates that this is FLV version 1.
//!
//! ### Reserved of 5 bits
//!
//! This shall be 0.
//!
//! ### Audio flag
//!
//! This indicates whether the audio data exists in this file.
//!
//! ### Reserved of 1 bit
//!
//! This shall be 0.
//!
//! ### Video flag
//!
//! This indicates whether the video data exists in this file.
//!
//! ### Data offset
//!
//! This is the offset to actual data.
//! That is, the size of this header.
//!
//! ## The FLV tag
//!
//! The FLV tag consits of following fields respectively:
//!
//! 1. Reserved (unsigned 2 **bits**)
//! 2. Filtered flag (unsigned 1 **bit**)
//! 3. Tag type (unsigned 5 **bits** number)
//! 4. Data size (unsigned 3 bytes number)
//! 5. Timestamp (unsigned 3 bytes number)
//! 6. Eextended timestamp (unsigned 1 byte number, in milliseconds)
//!    * This will be used when the timestamp has exceeded the limit for 3 bytes.
//!    * Note that this will be represented as upper 1 byte of the 4 bytes timestamp.
//! 7. Stream id (unsigned 3 bytes)
//! 8. Actual data
//!    * Audio
//!    * Video
//!    * ScriptData
//!
//! ### Reserved
//!
//! This shall be 0.
//!
//! ### Filtered flag
//!
//! This indicates whether the FLV bodies have been encrypted. 
//!
//! ### Tag type
//!
//! Tha tag type consists of following values:
//!
//! |Number|Tag type    |
//! | ---: | :--------- |
//! |8     |`Audio`     |
//! |9     |`Video`     |
//! |18    |`ScriptData`|
//!
//! Above numbers are identical with the message type id in the chunk message header.
//! That is, the `Audio` tag indicates the Audio of chunk data, the `Video` tag indicates the Video of chunk data, and the `ScriptData` indicates the Notify of chunk data.
//!
//! ### Data size
//!
//! The actual data size.
//! That is, this will be equal to what subturacts the total tag size to this header size.
//!
//! ### Timestamp
//!
//! The timestamp when this FLV tag was created.
//! This field must count based on the timestamp of first FLV tag relatively, and the timestamp of first FLV tag must 0.
//! This is counted in millisecond.
//!
//! ### Extended timestamp
//!
//! This will be used when the timestamp has exceeded 3 bytes limit, as the head 1 byte of 4 bytes timestamp.
//!
//! ### Stream id
//!
//! This must be input 0.
//!
//! ## The encryption tag
//!
//! The encryption tag consists of following fields respectively:
//!
//! 1. The additional header object
//! 2. The encryption tag header
//! 3. The filter params
//! 4. The encryption body
//!
//! ### The additional header object
//!
//! If you use encryption, the additional header object will be sent as the `Notify` chunk with "|AdditionalHeader" message, as the first FLV element, and just after "onMetaData" of ScriptData tag.
//! Note that the property is AMF's value.
//! The additional header object consists of following property:
//!
//! * The encryption header object
//!
//! #### The encryption header object
//!
//! The encryption header object will be sent as the ScriptData tag named as "Encryption".
//! And this consists of following properties:
//!
//! * Version (`Number`)
//! * Method (`String`)
//! * Some flags (`Number`)
//! * Params (The standard encoding parameters object)
//! * Sigformat/Signature (`String`/`LongString`)
//!
//! ##### Version
//!
//! This indicate the Encryption header's version.
//! The correspondence of the number to the version is following:
//!
//! |Number|Version                  |
//! | ---: | :---------------------- |
//! |1     |FMRMS v1.x products      |
//! |2     |Flash Access 2.0 products|
//!
//! ##### Method
//!
//! This shall be "Standard".
//!
//! ##### Some flags
//!
//! This shall be 0.
//!
//! ##### Standard encoding parameters object
//!
//! This consists of following properties:
//!
//! * Version (`Number`)
//! * EncryptionAlgorithm (`String`)
//! * EncryptionParams (The AES-CBC encryption parameters object)
//! * KeyInfo (The key information object)
//!
//! ###### Version
//!
//! This shall be 1.
//!
//! ###### EncryptionAlgorithm
//!
//! This shall be "AES-CBC".
//!
//! ###### The AES-CBC encryption parameters object
//!
//! This consists of following peoperty:
//!
//! * KeyLength (`Number`)
//!
//! This shall be 16. (128 bits)
//!
//! ###### The key information object
//!
//! This consists of following properties:
//!
//! * SubType (`String`)
//!
//! If the encryption header's version is 1, this will be input "APS", otherwise this will be input "FlashAccessv2".
//! APS is Adobe Policy Server.
//! Online key agreement negotiation protocol.
//! FlashAccessv2 is Flash Access 2.0 products.
//! Online key retrieval protocol.
//!
//! * Data
//!
//! If the encryption header's version is 1, this will be input the adobe policy server object, otherwise this will be input the flash access v2 object.
//!
//! The flash access v2 object consists of a `LongString` value named as "MetaData".
//! It's enceded by Base64, and will be used the DRM client to retrieve the decryption key.
//!
//! The adobe policy server object couldn't be found the specification due to no longer produced by conforming applications.
//!
//! ##### Sigformat/Signature (`String`/`LongString`)
//!
//! This hasn't written no document in the FLV file format specification.
//!
//! ### The encryption tag header
//!
//! The encryption tag header consists of following fields respectively:
//!
//! 1. Number of filters (unsigned 1 byte)
//! 2. Filter name (String)
//! 3. Length (unsigned 3 bytes)
//!
//! #### Number of filters
//!
//! This indicates the count of filters applied to the packet.
//! This shall be 1.
//!
//! #### Filter name (String)
//!
//! This is a string for filter name.
//! The name is following:
//!
//! * "Encryption" (if the version in the encryption header is 1)
//! * "SE" (otherwise)
//!
//! #### Length (unsigned 3 bytes)
//!
//! This indicates the length of the filter params (in byte).
//!
//! ### The filter params
//!
//! The filter params consist of following fields respectively:
//!
//! * Encryption filter params
//!
//! *or*
//!
//! * Selective encryption filter params
//!
//! #### The encryption filter params
//!
//! The encryption filter params consist of following field:
//!
//! 1. IV (array of unsigned 1 byte, total of 16 bytes)
//!
//! ##### IV
//!
//! The initialization vector to be used for AES-CBC encryption.
//!
//! #### The selective encryption filter params
//!
//! The selective encryption filter params consist of following fields respectively:
//!
//! 1. Encrypted AU (unsigned 1 **bit**)
//! 2. Reserved (unsigned 7 **bits**)
//!
//! ##### Encrypted AU
//!
//! This is indicates whether the packet is encrypted.
//!
//! ##### Reserved
//!
//! This shall be 0.
//!
//! 3. IV (array of unsigned 1 byte, total of 16 bytes)
//!
//! The initialization vector to be used for AES-CBC encryption.
//! This will be skipped unless the field of Encrypted AU is 1.
//!
//! ### The encryption body
//!
//! The encryption body consists of following fields respectively:
//!
//! 1. Content (array of unsigned 1 byte)
//! 2. Padding (array of unsigned 1 byte)
//!
//! #### Content
//!
//! If the field of Encrypted AU is 0, this will be input the plaintext.
//! If it is 1, this will be input the encrypted text.
//! This length is equal to the input text length.
//!
//! #### Padding (array of unsigned 1 byte)
//!
//! This is the padding string for encrypting the content.
//! This is also encrypted.
//! This length is equal to actual padding string length.
//! See RFC2630 for more detail about this.
//!
//! ## The audio tag
//!
//! The audio tag consists of following fields respectively:
//!
//! 1. The audio tag header.
//! 2. The audio data.
//!
//! ### The audio tag header.
//!
//! The audio tag header consits of following fields respectively:
//!
//! 1. Sound format (unsigned 4 **bits**)
//! 2. Sound rate (unsigned 2 **bits**)
//! 3. Sound size (unsigned 1 **bit**)
//! 4. Sound type (unsigned 1 **bit**)
//! 5. AAC packet type (unsigned 1 byte)
//!
//! #### Sound format
//!
//! This indicates the audio file format.
//! The correspondence of the number to the sound format is following:
//!
//! |Number|Sound format                |
//! | ---: | :------------------------- |
//! |0     |Linear PCM (native endian)  |
//! |1     |ADPCM                       |
//! |2     |MP3                         |
//! |3     |Linear PCM (little endian)  |
//! |4     |Nellymoser 16kHz mono       |
//! |5     |Nellymoser 8kHz mono        |
//! |6     |Nellymoser                  |
//! |7     |G.711 A-law logarithmic PCM |
//! |8     |G.711 mu-law logarithmic PCM|
//! |9     |Reserved                    |
//! |10    |AAC                         |
//! |11    |Speex                       |
//! |14    |MP3 8kHz                    |
//! |15    |Device-specific sound       |
//!
//! #### Sound rate
//!
//! This indicates the audio sampling rate.
//! The correspondence of the number to the sound rate is following:
//!
//! |Number|Sound rate (in kHz)|
//! | ---: | ----------------: |
//! |0     |5.5                |
//! |1     |11.0               |
//! |2     |22.0               |
//! |3     |44.0               |
//!
//! #### Sound size
//!
//! This indicates which the audio is either 8 bits or 16 bits.
//! The correspondence of the number to the sound size is following:
//!
//! |Number|Sound size (in bit)|
//! | ---: | ----------------: |
//! |0     |8                  |
//! |1     |16                 |
//!
//! #### Sound type (unsigned 1 **bit**)
//!
//! This indicates which the audio is either mono or stereo.
//! The correspondence of the number to the sound type is following:
//!
//! |Number|Sound type|
//! | ---: | :------- |
//! |0     |Mono      |
//! |1     |Stereo    |
//!
//! #### AAC packet type (unsigned 1 byte)
//!
//! This indicates which the audio is either AAC sequence header or AAC raw, if it is AAC.
//! Therefore this will be skipped unless the codec is AAC.
//! The correspondence of the number to the AAC packet type is following:
//!
//! |Number|AAC packet type    |
//! | ---: | :---------------- |
//! |0     |AAC sequence header|
//! |1     |AAC raw            |
//!
//! ### The audio data
//!
//! This has depended on the audio codec for its byte format.
//! See other documents about the audio codecs.
//!
//! ## The video tag
//!
//! The video tag consists of following fields respectively:
//!
//! 1. The video tag header
//! 2. The video data
//!
//! ### The video tag header
//!
//! The video tag header consists of following fields respectively:
//!
//! 1. Frame type (unsigned 4 **bits**)
//! 2. Codec id (unsigned 4 **bits**)
//! 3. AVC packet type (unsigned 1 byte)
//! 4. Composition time (unsigned 3 bytes)
//!
//! #### Frame type
//!
//! This indicates the video frame type.
//! The correspondence of the number to the frame type is following:
//!
//! |Number|Frame type              |
//! | ---: | :--------------------- |
//! |1     |key frame               |
//! |2     |inter frame             |
//! |3     |disposable inter frame  |
//! |4     |generated key frame     |
//! |5     |video info/command frame|
//!
//! #### Codec id (unsigned 4 **bits**)
//!
//! This indicates the codec for video data.
//! The correspondence of the number to the video codec is following:
//!
//! |Number|Video codec               |
//! | ---: | :----------------------- |
//! |2     |Sorenson H.263            |
//! |3     |Screen video              |
//! |4     |On2 VP6                   |
//! |5     |On2 VP6 with alpha channel|
//! |6     |Screen video version 2    |
//! |7     |AVC                       |
//!
//! #### AVC packet type (unsigned 1 byte)
//!
//! This indicate which the video is either AVC sequence header, AVC NALU or AVC end of sequence.
//! Therefore this will be skipped unless the codec is AVC.
//! The correspondence of the number to the AVC packet type is following:
//!
//! |Number|AVC packet type    |
//! | ---: | :---------------- |
//! |0     |AVC sequence header|
//! |1     |AVC NALU           |
//! |2     |AVC end of sequence|
//!
//! #### Composition time (signed 3 bytes)
//!
//! This indicates the time offset for the AVC codec.
//! Therefore this will be skipped unless the codec is AVC, and will be input 0 unless the AVC packet type is 1 (NALU).
//! See the ISO 14496-12 for more detail about the Composition time.
//!
//! ### The video data
//!
//! This has depended on the video codec for its byte format.
//! See other documents about the video codecs.
//!
//! ## The data tag
//!
//! The data tag consists of what is identical to the `MixedArray` of AMF sent as the `Notify` chunk currently.
//! That is, this consists of following fields respectively:
//!
//! 1. Name
//! 2. ScriptData
//!
//! ### Name
//!
//! "onMetaData" (`String`)
//!
//! ### ScriptData
//!
//! Following name/value pairs.
//! However these aren't input all necessarily.
//!
//! |Name           |AMF data type|
//! | :------------ | :---------- |
//! |audiocodecid   |`Number`     |
//! |audiodatarate  |`Number`     |
//! |audiodelay     |`Number`     |
//! |audiosamplerate|`Number`     |
//! |audiosamplesize|`Number`     |
//! |canSeekToEnd   |`Boolean`    |
//! |creationdate   |`String`     |
//! |duration       |`Number`     |
//! |filesize       |`Number`     |
//! |framerate      |`Number`     |
//! |height         |`Number`     |
//! |stereo         |`Boolean`    |
//! |videocodecid   |`Number`     |
//! |videodatarate  |`Number`     |
//! |width          |`Number`     |
//!
//! See `AmfData` for more detail about the AMF.
//!
//! [ISO 14496-12]: https://www.iso.org/standard/68960.html
//! [`AmfData`]: ./messages/enum.AmfData.html
//! [`RFC2630`]: https://tools.ietf.org/html/rfc2630
use std::{
    time::{
        Duration,
        SystemTime
    }
};
use crate::{
    encoders::*,
    messages::{
        ByteBuffer,
        MetaData
    }
};

/// # The FLV header
///
/// This consists of following data:
///
/// 1. Audio flag
/// 2. Video flag
/// 3. Version
/// 4. Data offset
///
/// ## Audio flag
///
/// This is the same as flv.rs.
///
/// ## Video flag
///
/// This is the same as flv.rs.
///
/// ## Version
///
/// This is the same as flv.rs.
///
/// ## Data offset
///
/// This is the same as flv.rs.
///
/// [flv.rs]: ../flv.rs.html
#[derive(Debug, Clone, Copy, Default)]
pub struct FlvHeader {
    has_audio: bool,
    has_video: bool,
    version: u8,
    offset: u32
}

impl FlvHeader {
    const SIGNATURE: &'static str = &"FLV";
}

/// # The tag type
///
/// The correspondence of the number to this enum is following:
///
/// |Number|TagType     |
/// | ---: | :--------- |
/// |8     |`Audio`     |
/// |9     |`Video`     |
/// |18    |`ScriptData`|
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum TagType {
    Audio = 8,
    Video = 9,
    ScriptData = 18
}

impl From<u8> for TagType {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `tag_type_id: u8`
    ///
    /// The number to indicate the tag type.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value of neither 8, 9 nor 18.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::TagType;
    ///
    /// let audio: TagType = (8 as u8).into();
    /// let video: TagType = (9 as u8).into();
    /// let script_data: TagType = (18 as u8).into();
    ///
    /// /* This will print `Audio`. */
    /// println!("{:?}", audio);
    /// /* This will print `Video`. */
    /// println!("{:?}", video);
    /// /* This will print `ScriptData`. */
    /// println!("{:?}", script_data);
    /// ```
    fn from(tag_type_id: u8) -> Self {
        use TagType::*;

        match tag_type_id {
            8 => Audio,
            9 => Video,
            18 => ScriptData,
            _ => panic!("Undefined tag type id!")
        }
    }
}

/// # The sound format
///
/// The correspondence of the number to this enum is following:
///
/// |Number|SoundFormat        |
/// | ---: | :---------------- |
/// |0     |`LinearNe`         |
/// |1     |`Adpcm`            |
/// |2     |`Mp3`              |
/// |3     |`LinearLe`         |
/// |4     |`NellymoserSixteen`|
/// |5     |`NellymoserEight`  |
/// |6     |`Nellymoser`       |
/// |7     |`G711A`            |
/// |8     |`G711mu`           |
/// |10    |`Aac`              |
/// |11    |`Speex`            |
/// |14    |`Mp3Eight`         |
/// |15    |`Other`            |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SoundFormat {
    LinearNe,
    Adpcm,
    Mp3,
    LinearLe,
    NellymoserSixteen,
    NellymoserEight,
    Nellymoser,
    G711A,
    G711mu,
    Aac = 10,
    Speex,
    Mp3Eight = 14,
    Other
}

impl From<u8> for SoundFormat {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `sound_format_id: u8`
    ///
    /// The number to indicate the sound format.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed either 9, 12, 13 or the value above 15.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::SoundFormat;
    ///
    /// let linear_ne: SoundFormat = (0 as u8).into();
    /// let adpcm: SoundFormat = (1 as u8).into();
    /// let mp3: SoundFormat = (2 as u8).into();
    /// let linear_le: SoundFormat = (3 as u8).into();
    /// let nerrymoser_sixteen: SoundFormat = (4 as u8).into();
    /// let nerrymoser_eight: SoundFormat = (5 as u8).into();
    /// let nerrymoser: SoundFormat = (6 as u8).into();
    /// let g711a: SoundFormat = (7 as u8).into();
    /// let g711mu: SoundFormat = (8 as u8).into();
    /// let aac: SoundFormat = (10 as u8).into();
    /// let speex: SoundFormat = (11 as u8).into();
    /// let mp3_eight: SoundFormat = (14 as u8).into();
    /// let other: SoundFormat = (15 as u8).into();
    ///
    /// /* This will print `LinearNe`. */
    /// println!("{:?}", linear_ne);
    /// /* This will print `Adpcm`. */
    /// println!("{:?}", adpcm);
    /// /* This will print `Mp3`. */
    /// println!("{:?}", mp3);
    /// /* This will print `LinearLe`. */
    /// println!("{:?}", linear_le);
    /// /* This will print `NerrymoserSixteen`. */
    /// println!("{:?}", nerrymoser_sixteen);
    /// /* This will print `NerrymoserEight`. */
    /// println!("{:?}", nerrymoser_eight);
    /// /* This will print `Nerrymoser`. */
    /// println!("{:?}", nerrymoser);
    /// /* This will print `G711a`. */
    /// println!("{:?}", g711a);
    /// /* This will print `G711mu`. */
    /// println!("{:?}", g711mu);
    /// /* This will print `Aac`. */
    /// println!("{:?}", aac);
    /// /* This will print `Speex`. */
    /// println!("{:?}", speex);
    /// /* This will print `Mp3Eight`. */
    /// println!("{:?}", mp3_eight);
    /// /* This will print `Other`. */ 
    /// println!("{:?}", other);
    /// ```
    fn from(sound_format_id: u8) -> Self {
        use SoundFormat::*;

        match sound_format_id {
            0 => LinearNe,
            1 => Adpcm,
            2 => Mp3,
            3 => LinearLe,
            4 => NellymoserSixteen,
            5 => NellymoserEight,
            6 => Nellymoser,
            7 => G711A,
            8 => G711mu,
            10 => Aac,
            11 => Speex,
            14 => Mp3Eight,
            15 => Other,
            _ => panic!("Undefined sound format id!")
        }
    }
}

/// # The sound rate
///
/// The correspondence of the number to this enum is following:
///
/// |Number|SoundRate      |
/// | ---: | :------------ |
/// |0     |`FivePointFive`|
/// |1     |`Eleven`       |
/// |2     |`TwnetyTwo`    |
/// |3     |`FortyFour`    |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SoundRate {
    FivePointFive,
    Eleven,
    TwentyTwo,
    FortyFour
}

impl From<u8> for SoundRate {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `sound_rate_id: u8`
    ///
    /// The number to indicate the sound rate.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value above 3.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::SoundRate;
    ///
    /// let five_point_five: SoundRate = (0 as u8).into();
    /// let eleven: SoundRate = (1 as u8).into();
    /// let twenty_two: SoundRate = (2 as u8).into();
    /// let fourty_four: SoundRate = (3 as u8).into();
    ///
    /// /* This will print `FivePointFive`. */
    /// println!("{:?}", five_point_five);
    /// /* This will print `Eleven`. */
    /// println!("{:?}", eleven);
    /// /* This will print `TwentyTwo`. */
    /// println!("{:?}", twenty_two);
    /// /* This will print `FourtyFour`. */
    /// println!("{:?}", fourty_four);
    /// ```
    fn from(sound_rate_id: u8) -> Self {
        use SoundRate::*;

        match sound_rate_id {
            0 => FivePointFive,
            1 => Eleven,
            2 => TwentyTwo,
            3 => FortyFour,
            _ => panic!("Undefined sound rate id!")
        }
    }
}

/// # The sound size
///
/// The correspondence of the number to this enum is following:
///
/// |Number|SoundSize|
/// | ---: | :------ |
/// |0     |`Eight`  |
/// |1     |`Sixteen`|
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SoundSize {
    Eight,
    Sixteen
}

impl From<u8> for SoundSize {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `sound_size_id: u8`
    ///
    /// The number to indicate the sound size.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value above 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::SoundSize;
    ///
    /// let eight: SoundSize = (0 as u8).into();
    /// let sixteen: SoundSize = (1 as u8).into();
    ///
    /// /* This will print `Eight`. */
    /// println!("{:?}", eight);
    /// /* This will print `Sixteen`. */
    /// println!("{:?}", sixteen);
    /// ```
    fn from(sound_size_id: u8) -> Self {
        use SoundSize::*;

        match sound_size_id {
            0 => Eight,
            1 => Sixteen,
            _ => panic!("Undefined sound size!")
        }
    }
}

/// # The sound type
///
/// The correspondence of the number to this enum is following:
///
/// |Number|SoundType|
/// | ---: | :------ |
/// |0     |`Mono`   |
/// |1     |`Stereo` |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SoundType {
    Mono,
    Stereo
}

impl From<u8> for SoundType {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `sound_type_id: u8`
    ///
    /// The number to indicate the sound type.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value above 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::SoundType;
    ///
    /// let mono: SoundType = (0 as u8).into();
    /// let stereo: SoundType = (1 as u8).into();
    ///
    /// /* This will print `Mono`. */
    /// println!("{:?}", mono);
    /// /* This will print `Stereo`. */
    /// println!("{:?}", stereo);
    /// ```
    fn from(sound_type_id: u8) -> Self {
        use SoundType::*;

        match sound_type_id {
            0 => Mono,
            1 => Stereo,
            _ => panic!("Undefined sound type id!")
        }
    }
}

/// # The AAC packet type
///
/// The correspondence of the number to this enum is following:
///
/// |Number|AacPacketType   |
/// | ---: | :------------- |
/// |0     |`SequenceHeader`|
/// |1     |`Raw`           |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum AacPacketType {
    SequenceHeader,
    Raw
}

impl From<u8> for AacPacketType {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `aac_packet_type_id: u8`
    ///
    /// The number to indicate the AAC packet type id.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value above 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::AacPacketType;
    ///
    /// let sequence_header: AacPacketType = (0 as u8).into();
    /// let raw: AacPacketType = (1 as u8).into();
    ///
    /// /* This will print `SequenceHeader`. */
    /// println!("{:?}", sequence_header);
    /// /* This will print `Raw`. */
    /// println!("{:?}", raw);
    /// ```
    fn from(aac_packet_type_id: u8) -> Self {
        use AacPacketType::*;

        match aac_packet_type_id {
            0 => SequenceHeader,
            1 => Raw,
            _ => panic!("Undefined AAC packet type!")
        }
    }
}

/// # The audio tag header
///
/// This consists of following data:
///
/// 1. Sond format
/// 2. Sound rate
/// 3. Sound type
/// 4. AAC packet type
///
/// ## Sound format
///
/// See the `SoundFormat`.
///
/// ## Sound rate
///
/// See the `Soundrate`.
///
/// ## Sound size
///
/// See the `SoundSize`.
///
/// ## Sound type
///
/// See the `SoundType`.
///
/// ## AAC packet type
///
/// See the `AacPacketType`.
///
/// [`SoundFormat`]: ./enum.SoundFormat.html
/// [`SoundRate`]: ./enum.SoundRate.html
/// [`SoundSize`]: ./enum.SoundSize.html
/// [`SoundType`]: ./enum.SoundType.html
/// [`AacPacketType`]: ./enum.AacPacketType.html
#[derive(Debug, Clone, Copy)]
pub struct AudioTagHeader {
    sound_format: SoundFormat,
    sound_rate: SoundRate,
    sound_size: SoundSize,
    sound_type: SoundType,
    aac_packet_type: Option<AacPacketType>
}

impl AudioTagHeader {
    fn real_len(&self) -> u32 {
        let aac_packet_type_size = if self.aac_packet_type.is_some() {
            1
        } else {
            0
        };

        1 + aac_packet_type_size
    }
}

/// # The frame type
///
/// The correspondence of the number to this enum is following:
///
/// |Number|FrameType        |
/// | ---: | :-------------- |
/// |1     |`Key`            |
/// |2     |`Inter`          |
/// |3     |`DisposableInter`|
/// |4     |`GeneratedKey`   |
/// |5     |`VideoInfo`      |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FrameType {
    Key = 1,
    Inter,
    DisposableInter,
    GeneratedKey,
    VideoInfo
}

impl From<u8> for FrameType {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `frame_type_id: u8`
    ///
    /// The number to indicate the frame type.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed either the value below 1 or the value above 5.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::FrameType;
    ///
    /// let key: FrameType = (1 as u8).into();
    /// let inter: FrameType = (2 as u8).into();
    /// let disposable_inter: FrameType = (3 as u8).into();
    /// let generated_key: FrameType = (4 as u8).into();
    /// let video_info: FrameType = (5 as u8).into();
    ///
    /// /* This will print `Key`. */
    /// println!("{:?}", key);
    /// /* This will print `Inter`. */
    /// println!("{:?}", inter);
    /// /* This will print `DisposableInter`. */
    /// println!("{:?}", disposable_inter);
    /// /* This will print `GenertedKey`. */
    /// println!("{:?}", generated_key);
    /// /* This will print `VideoInfo`. */
    /// println!("{:?}", video_info);
    /// ```
    fn from(frame_type_id: u8) -> Self {
        use FrameType::*;

        match frame_type_id {
            1 => Key,
            2 => Inter,
            3 => DisposableInter,
            4 => GeneratedKey,
            5 => VideoInfo,
            _ => panic!("Undefined frame type id!")
        }
    }
}

/// # The codec
///
/// The correspondence of the number to this enum is following:
///
/// |Number|Codec         |
/// | ---: | :----------- |
/// |2     |`SorensonH263`|
/// |3     |`ScreenVideo` |
/// |4     |`On2Vp6`      |
/// |5     |`On2Vp6a`     |
/// |6     |`ScreenVideo2`|
/// |7     |`Avc`         |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>` and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Codec {
    SorensonH263 = 2,
    ScreenVideo,
    On2Vp6,
    On2Vp6a,
    ScreenVideo2,
    Avc
}

impl From<u8> for Codec {
    /// Converts the `u8` value into this enum.
    ///
    /// # Parameters
    ///
    /// * `codec_id: u8`
    ///
    /// The number to indicate the video codec.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed either the value below 2 or the value above 7.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::Codec;
    ///
    /// let sorenson_h263: Codec = (2 as u8).into();
    /// let screen_video: Codec = (3 as u8).into();
    /// let on2_vp6: Codec = (4 as u8).into();
    /// let on2_vp6a: Codec = (5 as u8).into();
    /// let screen_video2: Codec = (6 as u8).into();
    /// let avc: Codec = (7 as u8).into();
    ///
    /// /* This will print `SorensonH263`. */
    /// println!("{:?}", sorenson_h263);
    /// /* This will print `ScreenVideo`. */
    /// println!("{:?}", screen_video);
    /// /* This will print `On2Vp6`. */
    /// println!("{:?}", on2_vp6);
    /// /* This will print `On2Vp6a`. */
    /// println!("{:?}", on2_vp6a);
    /// /* This will print `ScreenVideo2`. */
    /// println!("{:?}", screen_video2);
    /// /* This will print `Avc`. */
    /// println!("{:?}", avc);
    /// ```
    fn from(codec_id: u8) -> Self {
        use Codec::*;

        match codec_id {
            2 => SorensonH263,
            3 => ScreenVideo,
            4 => On2Vp6,
            5 => On2Vp6a,
            6 => ScreenVideo2,
            7 => Avc,
            _ => panic!("Undefined codec id!")
        }
    }
}

/// # The AVC packet type
///
/// This consists of following patterns:
/// The correspondence of the number to this enum is following:
///
/// |Number|AvcPakcetType   |
/// | ---: | :------------- |
/// |0     |`SequenceHeader`|
/// |1     |`Nalu`          |
/// |2     |`EndOfSequence` |
///
/// This enum and the `u8` value can convert into each other because this has implemented the `From<u8>`, and has set the `#[repr(u8)]` attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum AvcPacketType {
    SequenceHeader,
    Nalu,
    EndOfSequence
}

impl From<u8> for AvcPacketType {
    /// Converts the `u8` value into this enum.
    /// 
    /// # Parameters
    ///
    /// * `avc_packet_type_id: u8`
    ///
    /// The number to indicate the AVC packet type.
    ///
    /// # Panics
    ///
    /// This will emit the `panic!` if is passed the value above 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::AvcPacketType;
    ///
    /// let sequence_header: AvcPacketType = (0 as u8).into();
    /// let nalu: AvcPacketType = (1 as u8).into();
    /// let end_of_sequence: AvcPacketType = (2 as u8).into();
    ///
    /// /* This will print `SequenceHeader`. */
    /// println!("{:?}", sequence_header);
    /// /* This will print `Nalu`. */
    /// println!("{:?}", nalu);
    /// /* This will print `EndOfSequence`. */
    /// println!("{:?}", end_of_sequence);
    /// ```
    fn from(avc_packet_type_id: u8) -> Self {
        use AvcPacketType::*;

        match avc_packet_type_id {
            0 => SequenceHeader,
            1 => Nalu,
            2 => EndOfSequence,
            _ => panic!("Undefined avc packet type id!")
        }
    }
}

/// # The video tag header
///
/// This consists of following data:
///
/// 1. Frame type
/// 2. Codec
/// 3. AVC packet type (optional)
/// 4. Composition time (optional)
///
/// ## Frame type
///
/// See the `FrameType`
///
/// ## Codec
///
/// See the `Codec`.
///
/// ## AVC packet type
///
/// See the `AvcPacketType`.
///
/// ## Composition time
///
/// This is the same as flv.rs.
///
/// [`FrameType`]: ./enum.FrameType.html
/// [`Codec`]: ./enum.Codec.html
/// [`AvcPakcetType`]: ./enum.AvcPacketType.html
/// [flv.rs]: ../flv.rs.html
#[derive(Debug, Clone, Copy)]
pub struct VideoTagHeader {
    frame_type: FrameType,
    codec: Codec,
    avc_packet_type: Option<AvcPacketType>,
    composition_time: Option<Duration>
}

impl VideoTagHeader {
    fn real_len(&self) -> u32 {
        let avc_packet_type_size = if self.avc_packet_type.is_some() {
            1
        } else {
            0
        };
        let composition_time_size = if self.composition_time.is_some() {
            3
        } else {
            0
        };

        1 + avc_packet_type_size + composition_time_size
    }
}

/// # The encryption tag header
///
/// This consists of following data:
///
/// 1. Number of filters
/// 2. Length of the filter params
/// 3. Filter name.
///
/// ## Number of filters
///
/// This is the same as flv.rs.
///
/// ## Length of the filter params
///
/// This is the same as flv.rs.
///
/// ## Filter name
///
/// This is the same as flv.rs.
///
/// [flv.rs]: ../flv.rs.html
#[derive(Debug, Clone)]
pub struct EncryptionTagHeader {
    filters_count: u8,
    length: u32,
    filter_name: String
}

impl EncryptionTagHeader {
    fn real_len(&self) -> u32 {
        4 + self.filter_name.len() as u32
    }
}

/// # The filter parameters
///
/// This consists of following patterns:
///
/// 1. `Encryption`
/// 2. `SelectiveEncryption`
///
/// ## Encryption
///
/// This is the same as flv.rs.
///
/// ## SelectiveEncryption
///
/// This is the same as flv.rs.
///
/// [flv.rs]: ../flv.rs.html
#[derive(Debug, Clone)]
pub enum FilterParams {
    Encryption(Vec<u8>),
    SelectiveEncryption(bool, Option<Vec<u8>>)
}

impl FilterParams {
    fn real_len(&self) -> u32 {
        match self {
            &FilterParams::Encryption(_) => 16,
            &FilterParams::SelectiveEncryption(is_encrypted, _) => if is_encrypted {
                24
            } else {
                8
            }
        }
    }
}

/// # The encryption tag
///
/// This consists of following data:
///
/// 1. Filter parameters
/// 2. Encryption tag header
///
/// ## Filter parameters
///
/// See the `FilterParams`.
///
/// ## Encryption tag header
///
/// See the `EncryptionTagHeader`.
///
/// [`FilterParams`]: ./enum.FilterParams.html
/// [`EncryptionTagHeader`]: ./struct.EncryptionTagHeader.html
#[derive(Debug, Clone)]
pub struct EncryptionTag {
    filter_params: FilterParams,
    encryption_tag_header: EncryptionTagHeader
}

impl EncryptionTag {
    fn real_len(&self) -> u32 {
        self.filter_params.real_len() + self.encryption_tag_header.real_len()
    }
}

/// # The audio tag
///
/// This consists of following data:
///
/// 1. Encryption tag (optional)
/// 2. Audio tag header
/// 3. Audio data
///
/// ## Encryption tag
///
/// See the `EncryptionTag`.
///
/// ## Audio tag header
///
/// See the `AudioTagHeader`.
///
/// [`EncryptionTag`]: ./struct.EncryptionTag.html
/// [`AudioTagHeader`]: ./struct.AudioTagHeader.html
#[derive(Debug, Clone)]
pub struct AudioTag {
    encryption_tag: Option<EncryptionTag>,
    audio_tag_header: AudioTagHeader,
    bytes: Vec<u8>
}

impl AudioTag {
    fn real_len(&self) -> u32 {
        let mut real_len = self.audio_tag_header.real_len() + self.bytes.len() as u32;

        if let &Some(ref encryption_tag) = &self.encryption_tag {
            real_len += encryption_tag.real_len();
        }

        real_len
    }
}

impl From<Vec<u8>> for AudioTag {
    /// Converts the bytes into this type.
    /// The first several bytes are required to correspond to the audio tag header's format.
    /// See the `AudioTagHeader` for more detail about the audio tag header.
    ///
    /// # Parameters
    ///
    /// * `bytes: Vec<u8>`
    ///
    /// The audio data bytes to convert into this type.
    ///
    /// # Panics
    ///
    /// If the first several bytes didn't correspond to the audio tag header's format, this will emit the `panic!`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::{
    ///     AudioTag,
    ///     SoundFormat,
    ///     SoundRate,
    ///     SoundSize,
    ///     SoundType
    /// };
    ///
    /// let mut bytes: Vec<u8> = Vec::new();
    /// /*
    ///  * The first byte consists of following formats respectively:
    ///  *
    ///  * 1. Sound format (4 **bits**)
    ///  * 2. Sound rate (2 **bits**)
    ///  * 3. Sound size (1 **bit**)
    ///  * 4. Sound type (1 **bit**)
    /// */
    /// let sound_format = (SoundFormat::Mp3 as u8) << 4;
    /// let sound_rate = (SoundRate::FortyFour as u8) << 2;
    /// let sound_size = (SoundSize::Sixteen as u8) << 1;
    /// let sound_type = SoundType::Stereo as u8;
    ///
    /// bytes.push(sound_format | sound_rate | sound_size | sound_type);
    ///
    /// /* Below is irresponsible bytes due to unconsidered of actual audio format. */
    /// for _ in 0..320 {
    ///     bytes.push(0);
    /// }
    ///
    /// let audio_tag: AudioTag = bytes.into();
    ///
    /// /*
    ///  * This will print following format:
    ///  * AudioTag {
    ///  *     encryption_tag: None,
    ///  *     audio_tag_header: AudioTagHeader {
    ///  *         sound_format: Mp3,
    ///  *         sound_rate: FortyFour,
    ///  *         sound_size: Sixteen,
    ///  *         sound_type: Stereo,
    ///  *         aac_packet_type: None
    ///  *     },
    ///  *     bytes: [...]
    ///  * }
    /// */
    /// println!("{:?}", audio_tag);
    /// ```
    ///
    /// [`AudioTagHeader`]: ./struct.AudioTagHeader.html
    fn from(mut bytes: Vec<u8>) -> Self {
        let mut offset = usize::default();
        let byte_audio_tag_header = bytes[offset];
        let sound_format: SoundFormat = ((byte_audio_tag_header & 0xf0) >> 4).into();
        let sound_rate: SoundRate = ((byte_audio_tag_header & 0x0c) >> 2).into();
        let sound_size: SoundSize = ((byte_audio_tag_header & 0x02) >> 1).into();
        let sound_type: SoundType = (byte_audio_tag_header & 0x01).into();

        offset += 1;

        let aac_packet_type: Option<AacPacketType> = if let SoundFormat::Aac = sound_format {
            let aac_packet_type_id = bytes[offset];

            offset += 1;
            Some(aac_packet_type_id.into())
        } else {
            None
        };
        let audio_tag_header = AudioTagHeader {
            sound_format,
            sound_rate,
            sound_size,
            sound_type,
            aac_packet_type
        };

        bytes = bytes[offset..].to_vec();
        AudioTag {
            encryption_tag: None,
            audio_tag_header,
            bytes
        }
    }
}

/// # The video tag
///
/// This consists of following data:
///
/// 1. Encryption tag (optional)
/// 2. Video tag header
/// 3. Video data
///
/// ## Encryption tag
///
/// See the `EncryptionTag`.
///
/// ## Video tag header
///
/// See the `VideoTagHeader`.
///
/// [`EncryptionTag`]: ./struct.EncryptionTag.html
/// [`VideoTagHeader`]: ./struct.VideoTagHeader.html
#[derive(Debug, Clone)]
pub struct VideoTag {
    encryption_tag: Option<EncryptionTag>,
    video_tag_header: VideoTagHeader,
    bytes: Vec<u8>
}

impl VideoTag {
    fn real_len(&self) -> u32 {
        let mut real_len = self.video_tag_header.real_len() + self.bytes.len() as u32;

        if let &Some(ref encryption_tag) = &self.encryption_tag {
            real_len += encryption_tag.real_len();
        }

        real_len
    }
}

impl From<Vec<u8>> for VideoTag {
    /// Converts the bytes into this type.
    /// The first several bytes are required to correspond to the video tag header's format.
    /// See the `VideoTagHeader` for more detail about the video tag header.
    ///
    /// # Parameters
    ///
    /// * `bytes: Vec<u8>`
    ///
    /// The video data bytes to convert this type.
    ///
    /// # Panics
    ///
    /// If the first several bytes didn't correspond to the video tag header's format, this will emit the `panic!`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::flv::{
    ///     Codec,
    ///     FrameType,
    ///     VideoTag
    /// };
    ///
    /// let mut bytes: Vec<u8> = Vec::new();
    /// /*
    ///  * The first byte consists of following formats respectively:
    ///  *
    ///  * 1. Frame type (4 **bits**)
    ///  * 2. Codec id (4 **bits**)
    /// */
    /// let frame_type = (FrameType::DisposableInter as u8) << 4;
    /// let codec_id = Codec::SorensonH263 as u8;
    ///
    /// bytes.push(frame_type | codec_id);
    ///
    /// /* Below is irresponsible bytes due to unconsidered of actual audio format. */
    /// for _ in 0..1000 {
    ///     bytes.push(0);
    /// }
    ///
    /// let video_tag: VideoTag = bytes.into();
    ///
    /// /*
    ///  * This will print following format:
    ///  * VideoTag {
    ///  *     encryption_tag: None,
    ///  *     video_tag_header: {
    ///  *         frame_type: DisposableInter,
    ///  *         codec: Mp4,
    ///  *         avc_packet_type: None,
    ///  *         composition_time: None
    ///  *     },
    ///  *     bytes: [...]
    ///  * }
    /// */
    /// println!("{:?}", video_tag);
    /// ```
    ///
    /// [`VideoTagHeader`]: ./struct.VideoTagHeader.html
    fn from(mut bytes: Vec<u8>) -> Self {
        let mut offset = usize::default();
        let byte_video_tag_header = bytes[offset];
        let frame_type: FrameType = ((byte_video_tag_header & 0xf0) >> 4).into();
        let codec: Codec = (byte_video_tag_header & 0x0f).into();

        offset += 1;

        let avc_packet_type: Option<AvcPacketType> = if let Codec::Avc = codec {
            let avc_packet_type_id = bytes[offset];

            offset += 1;
            Some(avc_packet_type_id.into())
        } else {
            None
        };
        let composition_time = if let Codec::Avc = codec {
            if let &Some(AvcPacketType::Nalu) = &avc_packet_type {
                let bytes_composition_time = &bytes[offset..3];
                let mut tmp: [u8; 4] = [0; 4];

                for i in 0..bytes_composition_time.len() {
                    tmp[i + 1] = bytes_composition_time[i];
                }

                offset += 3;
                Some(Duration::from_millis(u32::from_be_bytes(tmp) as u64))
            } else {
                offset += 3;
                Some(Duration::default())
            }
        } else {
            None
        };
        let video_tag_header = VideoTagHeader {
            frame_type,
            codec,
            avc_packet_type,
            composition_time
        };

        bytes = bytes[offset..].to_vec();
        VideoTag {
            encryption_tag: None,
            video_tag_header,
            bytes
        }
    }
}

/// # The Data tag
///
/// This consists of following data respectively:
///
/// 1. Encryption tag (optional)
/// 2. Script data bytes
///
/// ## Encryption tag
///
/// See the `EncryptionTag`.
///
/// [`EncryptionTag`]: ./struct.EncryptionTag.html
#[derive(Debug, Clone)]
pub struct DataTag {
    encryption_tag: Option<EncryptionTag>,
    bytes: Vec<u8>
}

impl DataTag {
    fn real_len(&self) -> u32 {
        let mut real_len = self.bytes.len() as u32;

        if let &Some(ref encryption_tag) = &self.encryption_tag {
            real_len += encryption_tag.real_len();
        }

        real_len
    }
}

impl From<MetaData> for DataTag {
    /// Converts the FLV metadata into this type.
    /// First, puts "onMetaData" (`String`) as the property name into the buffer.
    /// Then puts the metadata into the buffer after converting into AMF's mixed array.
    ///
    /// # Parameters
    ///
    /// * `meta_data: MetaData`
    ///
    /// The metadata to convert into this type.
    /// See the `MetaData` for more detail about the MetaData type, and see flv.rs for more detail about the properties of metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use sheave::{
    ///     flv::DataTag,
    ///     messages::MetaData
    /// };
    ///
    /// let mut meta_data = MetaData::new();
    ///
    /// meta_data.set_stereo(Some(true));
    /// meta_data.set_audio_codec_id(Some(2 as f64));
    /// meta_data.set_audio_data_rate(Some(320 as f64));
    /// meta_data.set_audio_sample_rate(Some(44100 as f64));
    /// meta_data.set_audio_sample_size(Some(16 as f64));
    /// meta_data.set_video_codec_id(Some(2 as f64));
    /// meta_data.set_video_data_rate(Some(1000 as f64));
    /// meta_data.set_frame_rate(Some(30 as f64));
    /// meta_data.set_width(Some(1920 as f64));
    /// meta_data.set_height(Some(1080 as f64));
    ///
    /// let data_tag: DataTag = meta_data.into();
    ///
    /// /*
    ///  * This will print following format:
    ///  * DataTag {
    ///  *     encryption_tag: None,
    ///  *     bytes: [2, "onMetaData", {...}]
    ///  * }
    /// */
    /// println!("{:?}", data_tag);
    /// ```
    ///
    /// [`MetaData`]: ../messages/struct.MetaData.html
    /// [flv.rs]: ../flv.rs.html
    fn from(meta_data: MetaData) -> Self {
        let mut buffer = ByteBuffer::new(Vec::new());

        buffer.encode_amf_string("onMetaData".to_string());
        buffer.encode_amf_mixed_array(meta_data.into());

        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(buffer.bytes().as_slice());

        DataTag {
            encryption_tag: None,
            bytes
        }
    }
}

/// # The FLV data
///
/// This consists of following patterns:
///
/// 1. `Audio`
/// 2. `Video`
/// 3. `Data`
///
/// ## Audio
///
/// See the `AudioTag`.
///
/// ## Video
///
/// See the `VideoTag`.
///
/// ## Data
///
/// See the `DataTag`.
///
/// [`AudioTag`]: ./struct.AudioTag.html
/// [`VideoTag`]: ./struct.VideoTag.html
/// [`DataTag`]: ./struct.DataTag.html
#[derive(Debug, Clone)]
pub enum FlvData {
    Audio(AudioTag),
    Video(VideoTag),
    Data(DataTag)
}

impl FlvData {
    fn has_encryption_tag(&self) -> bool {
        match self {
            &FlvData::Audio(ref audio_tag) => audio_tag.encryption_tag.is_some(),
            &FlvData::Video(ref video_tag) => video_tag.encryption_tag.is_some(),
            &FlvData::Data(ref data_tag) => data_tag.encryption_tag.is_some()
        }
    }

    fn real_len(&self) -> u32 {
        match self {
            &FlvData::Audio(ref audio_tag) => audio_tag.real_len(),
            &FlvData::Video(ref video_tag) => video_tag.real_len(),
            &FlvData::Data(ref data_tag) => data_tag.real_len()
        }
    }
}

/// # The FLV tag
///
/// This consists of following data:
///
/// 1. Filtered flag
/// 2. Tag type
/// 3. Data size
/// 4. Stream id
/// 5. Timestamp
/// 6. FLV data
///
/// ## Filtered flag
///
/// This is the same as flv.rs.
///
/// ## Tag type
///
/// See the `TagType`.
///
/// ## Data size
///
/// This is the same as flv.rs.
///
/// ## Stream id
///
/// This is the same as flv.rs.
///
/// ## FLV data
///
/// See the `FlvData`.
///
/// [flv.rs]: ../flv.rs.html
/// [`TagType`]: ./enum.TagType.html
/// [`FlvData`]: ./enum.FlvData.html
#[derive(Debug, Clone)]
pub struct FlvTag {
    is_filtered: bool,
    tag_type: TagType,
    data_size: u32,
    stream_id: u32,
    timestamp: Duration,
    data: FlvData
}

impl FlvTag {
    fn real_len(&self) -> u32 {
        11 + self.data.real_len()
    }
}

/// # The FLV file bodies
///
/// This consists of following data:
///
/// 1. Size of previous tag
/// 2. FLV tag
///
/// ## Size of Previous tag.
///
/// This is the same as flv.rs.
///
/// ## FLV tag
///
/// See the `FlvTag`.
///
/// [flv.rs]: ../flv.rs.html
/// [`FlvTag`]: ./struct.FlvTag.html
#[derive(Debug, Clone)]
pub struct FlvBody {
    previous_size: u32,
    flv_tag: FlvTag
}

/// # The FLV
///
/// This will be used to store the FLV file format converted from the bytes.
/// This consists of following data:
///
/// 1. FLV header
/// 2. Timestamp what created this struct
/// 3. FLV bodies
///
/// ## FLV header
///
/// See the `FlvHeader`.
///
/// # FLV bodies
///
/// See the `FlvBody`.
///
/// [`FlvHeader`]: ./struct.FlvHeader.html
/// [`FlvBody`]: ./struct.FlvBody.html
#[derive(Debug, Clone, Default)]
pub struct Flv {
    flv_header: FlvHeader,
    created: Duration,
    body: Vec<FlvBody>
}

impl Flv {
    /// Appends the metadata into this struct.
    ///
    /// # Parameters
    ///
    /// * `meta_data: MetaData`
    ///
    /// The metadata to append into this struct.
    /// See the `MetaData` for more detail about this type.
    ///
    /// [`MetaData`]: ../messages/struct.MetaData.html
    pub fn append_meta_data(&mut self, meta_data: MetaData) {
        self.flv_header.has_audio = meta_data.get_audio_codec_id().is_some();
        self.flv_header.has_video = meta_data.get_video_codec_id().is_some();

        let previous_size = self.body.last().map_or(
            0,
            |last_flv_body| last_flv_body.previous_size + last_flv_body.flv_tag.real_len()
        );
        let data = FlvData::Data(meta_data.into());
        let is_filtered = data.has_encryption_tag();
        let data_size = data.real_len();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap() - self.created;

        self.body.push(
            FlvBody {
                previous_size,
                flv_tag: FlvTag {
                    is_filtered,
                    tag_type: TagType::ScriptData,
                    data_size,
                    stream_id: 0,
                    timestamp,
                    data
                }
            }
        );
    }

    /// Appends the bytes of audio data into this struct.
    ///
    /// # Parameters
    ///
    /// * `bytes: Vec<u8>`
    ///
    /// The bytes to append into this struct.
    pub fn append_audio(&mut self, bytes: Vec<u8>) {
        let previous_size = self.body.last().map_or(
            0,
            |last_flv_body| last_flv_body.previous_size + last_flv_body.flv_tag.real_len()
        );
        let data = FlvData::Audio(bytes.into());
        let is_filtered = data.has_encryption_tag();
        let data_size = data.real_len();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap() - self.created;

        self.body.push(
            FlvBody {
                previous_size,
                flv_tag: FlvTag {
                    is_filtered,
                    tag_type: TagType::Audio,
                    data_size,
                    stream_id: 0,
                    timestamp,
                    data
                }
            }
        );
    }

    /// Appends the bytes of video data into this struct.
    ///
    /// # Parameters
    ///
    /// * `bytes: Vec<u8>`
    ///
    /// The bytes to append into this struct.
    pub fn append_video(&mut self, bytes: Vec<u8>) {
        let previous_size = self.body.last().map_or(
            0,
            |last_flv_body| last_flv_body.previous_size + last_flv_body.flv_tag.real_len()
        );
        let data = FlvData::Video(bytes.into());
        let is_filtered = data.has_encryption_tag();
        let data_size = data.real_len();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap() - self.created;

        self.body.push(
            FlvBody {
                previous_size,
                flv_tag: FlvTag {
                    is_filtered,
                    tag_type: TagType::Video,
                    data_size,
                    stream_id: 0,
                    timestamp,
                    data
                }
            }
        );
    }
}
