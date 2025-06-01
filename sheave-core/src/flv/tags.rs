//! # The FLV tags
//!
//! FLV bodies consist of following tags.
//!
//! * [`Audio`] tag
//! * [`Video`] tag
//! * [`ScriptData`] tag
//!
//! And any tag has following common header.
//!
//! 1. Reserved bits for the Flash Media Server. (2 bits. However this is fiexed to 0.)
//! 2. Whether packets are filtered (1 bit)
//! 3. Tag type (5 bits. these are same as RTMP's message types. See [`MessageType`].)
//!   * Audio (8)
//!   * Video (9)
//!   * ScriptData (18)
//! 4. Actual tag size (subtracts this header size from the total)
//! 5. Timestamp (24 bits)
//! 6. Timestamp(extended) (8 bits)
//! 7. Message Stream ID (24 bits. However this is fixed to 0.)
//! 8. Actual tag data (Same size as the DataSize field)
//!
//! Note that currently the RTMP tools aren't checking whether flv data are encrypted.
//!
//! ## [`Audio`]
//!
//! The audio tag consists of:
//!
//! 1. SoundFormat / Audio Codec (4 bits. See [Sound Format](#sound-format))
//! 2. SoundRate / Sampling Rate (2 bits)
//!    * 5.5 kHz (0)
//!    * 11 kHz (1)
//!    * 22 kHz (2)
//!    * 44 kHz (3)
//! 3. SoundSize / Sampling bit width (1 bit)
//!    * 8 bits (0)
//!    * 16 bits (1)
//! 4. SoundType / Mono or Stereo (1 bit)
//!    * Mono (0)
//!    * Stereo (0)
//! 5. AACPacketType (8 bits if sound format is the AAC)
//!    * Sequence header (0)
//!    * Raw (1)
//! 6. AudioData (Arbitrary size)
//!
//! ### Sound Format
//!
//! The SoundFormat field corresponds to:
//!
//! |Value|Sound Format|
//! | -: | :- |
//! |`0`|Linear PCM (Native Endian)|
//! |`1`|ADPCM|
//! |`2`|MP3|
//! |`3`|Linear PCM (Little Endian)|
//! |`4`|Nellymoser (16 kHz, mono)|
//! |`5`|Nellymoser (8 kHz, mono)|
//! |`6`|Nellymoser|
//! |`7`|G.711 (A-law)|
//! |`8`|G.711 (mu-law)|
//! |`9`|reserved|
//! |`10`|AAC|
//! |`11`|Speex|
//! |`14`|MP3 (8 kHz)|
//! |`15`|Device-specific format|
//!
//! ## [`Video`]
//!
//! The video tag consists of:
//!
//! 1. FrameType (4 bits)
//!    * key frame (1, for AVC which is a seekable frame.)
//!    * inter frame (2, for AVC which is a non-seekable frame.)
//!    * disposable inter frame (3, for H.263.)
//!    * generated key frame (4, reserved for server use.)
//!    * video info/command frame (5)
//! 2. CodecID / Video Codec (4 bits. See [Video Codec](#video-codec))
//! 3. AVCPacketType (8 bits if video codec is the AVC.)
//!    * Sequence header (0)
//!    * NALU (1)
//!    * End of sequence header (2)
//! 4. CompositionTime (**signed** 24 bits if video codec is the AVC.)
//! 5. VideoData (Arbitrary size)
//!
//! ### Video Codec
//!
//! The CodecID field corresponds to:
//!
//! |Value|Codec|
//! | -: | :- |
//! |`2`|H.263|
//! |`3`|Screen Video|
//! |`4`|VP6|
//! |`5`|VP6 with alpha channel|
//! |`6`|Screen Video (v2)|
//! |`7`|AVC|
//!
//! ## [`ScriptData`]
//!
//! Currently, this is used to contain following pair:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |name|[`String`]|`"onMetaData"`|
//! |value|[`EcmaArray`]|See [Meta Data](#meta-data)|
//!
//! ### Meta Data
//!
//! This contains informations about audio/video configuration in FLV.
//! Note all of following pairs aren't necessarily contained.
//!
//! |Name|AMF Type|Value|
//! | :- | :- | :- |
//! |`audiocodecid`|[`Number`]|See [`Audio`].|
//! |`audiodatarate`|[`Number`]|An audio's bitrate.|
//! |`audiodelay`|[`Number`]|A time to indicate overhead by encoding/decoding in seconds.|
//! |`audiosamplerate`|[`Number`]|An audio's sampling frequency.|
//! |`audiosamplesize`|[`Number`]|An audio's sampling bitwidth.|
//! |`canSeekToEnd`|[`Boolean`]|Whether the last video frame is key frame.|
//! |`creationdate`|[`String`]|A datetime this FLV data is created. (Probably the ISO 8601 format)|
//! |`duration`|[`Number`]|A total duration range of this FLV data in seconds.|
//! |`filesize`|[`Number`]|A total file size of this FLV data in bytes.|
//! |`framerate`|[`Number`]|A video's framerate.|
//! |`height`|[`Number`]|A video frame's vertical size in pixels.|
//! |`stereo`|[`Boolean`]|Whether audio is sampled as stereo.|
//! |`videocodecid`|[`Number`]|See [`Video`]|
//! |`videodatarate`|[`Number`]|A video's bitrate.|
//! |`width`|[`Number`]|A video frame's horizonal size in pixels.|
//!
//! [`Audio`]: AudioTag
//! [`Video`]: VideoTag
//! [`ScriptData`]: ScriptDataTag
//! [`MessageType`]: crate::messages::headers::MessageType
//! [`Number`]: crate::messages::amf::v0::Number
//! [`Boolean`]: crate::messages::amf::v0::Boolean
//! [`String`]: crate::messages::amf::v0::AmfString
//! [`Object`]: crate::messages::amf::v0::Object
//! [`EcmaArray`]: crate::messages::amf::v0::EcmaArray
mod audio;
mod video;
mod script_data;

use std::time::Duration;
pub use self::{
    audio::*,
    video::*,
    script_data::*
};

/// The length of metadata which are common to every FLV tag.
pub const METADATA_LEN: usize = 11;
/// The Message ID which is written into FLV metadata (however this will never be read).
pub const DEFAULT_MESSAGE_ID: u32 = 0;

/// Representation of TagType bits of the FLV tag.
///
/// Variants correspond to respectively following types.
///
/// |Pattern|Number|
/// | :- | :- |
/// |`Audio`|`8`|
/// |`Video`|`9`|
/// |`ScriptData`|`18`|
/// |`Other`|other numbers|
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    Audio = 8,
    Video = 9,
    ScriptData = 18,
    Other = 31
}

impl From<u8> for TagType {
    fn from(tag_type: u8) -> Self {
        use TagType::*;

        match tag_type {
            8 => Audio,
            9 => Video,
            18 => ScriptData,
            _ => Other
        }
    }
}

impl From<TagType> for u8 {
    fn from(tag_type: TagType) -> Self {
        tag_type as u8
    }
}

/// The FLV tag element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlvTag {
    tag_type: TagType,
    timestamp: Duration,
    data: Vec<u8>
}

impl FlvTag {
    /// Constructs a FlvTag.
    pub fn new(tag_type: TagType, timestamp: Duration, data: Vec<u8>) -> Self {
        Self {
            tag_type,
            timestamp,
            data
        }
    }

    /// Gets the tag type.
    pub fn get_tag_type(&self) -> TagType {
        self.tag_type
    }

    /// Gets the timestamp.
    pub fn get_timestamp(&self) -> Duration {
        self.timestamp
    }

    /// Gets a message data. 
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}
