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
//!   * ScriptData (16)
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
//! | :- | :- |
//! |0|Linear PCM (Native Endian)|
//! |1|ADPCM|
//! |2|MP3|
//! |3|Linear PCM (Little Endian)|
//! |4|Nellymoser (16 kHz, mono)|
//! |5|Nellymoser (8 kHz, mono)|
//! |6|Nellymoser|
//! |7|G.711 (A-law)|
//! |8|G.711 (mu-law)|
//! |9|reserved|
//! |10|AAC|
//! |11|Speex|
//! |14|MP3 (8 kHz)|
//! |15|Device-specific format|
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
//! | :- | :- |
//! |2|H.263|
//! |3|Screen Video|
//! |4|VP6|
//! |5|VP6 with alpha channel|
//! |6|Screen Video (v2)|
//! |7|AVC|
//!
//! ## [`ScriptData`]
//!
//! Currently, this is used to contain following pair:
//!
//! |Field|AMF Type|Value|
//! | :- | :- | :- |
//! |name|`String`|`"onMetaData"`|
//! |value|`EcmaArray`|See [Meta Data](#meta-data)|
//!
//! ### Meta Data
//!
//! This contains informations about audio/video configuration in FLV.
//! Note all of following pairs aren't necessarily contained.
//!
//! |Name|AMF Type|Value|
//! | :- | :- | :- |
//! |audiocodecid|`Number`|See [`Audio`].|
//! |audiodatarate|`Number`|An audio's bitrate.|
//! |audiodelay|`Number`|A time to indicate overhead by encoding/decoding in seconds.|
//! |audiosamplerate|`Number`|An audio's sampling frequency.|
//! |audiosamplesize|`Number`|An audio's sampling bitwidth.|
//! |canSeekToEnd|`Boolean`|Whether the last video frame is key frame.|
//! |creationdate|`String`|A datetime this FLV data is created. (Probably the ISO 8601 format)|
//! |duration|`Number`|A total duration range of this FLV data in seconds.|
//! |filesize|`Number`|A total file size of this FLV data in bytes.|
//! |framerate|`Number`|A video's framerate.|
//! |height|`Number`|A video frame's vertical size in pixels.|
//! |stereo|`Boolean`|Whether audio is sampled as stereo.|
//! |videocodecid|`Number`|See [`Video`]|
//! |videodatarate|`Number`|A video's bitrate.|
//! |width|`Number`|A video frame's horizonal size in pixels.|
//!
//! [`Audio`]: AudioTag
//! [`Video`]: VideoTag
//! [`ScriptData`]: ScriptDataTag
//! [`MessageType`]: crate::messages::headers::message::message_type::MessageType
mod audio;
mod video;
mod script_data;

use std::{
    io::Result as IOResult,
    time::Duration
};
use crate::{
    ByteBuffer,
    Decoder,
    Encoder
};
use super::unknown_tag;
pub use self::{
    audio::*,
    video::*,
    script_data::*
};

/// The tag body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InnerTag {
    Audio(AudioTag),
    Video(VideoTag),
    ScriptData(ScriptDataTag)
}

/// Representation of TagType bits of the FLV tag.
///
/// Variants correspond to respectively following types.
///
/// |Pattern|Number|
/// | :- | :- |
/// |`Audio`|`8`|
/// |`Video`|`9`|
/// |`ScriptData`|`16`|
/// |`Other`|other numbers|
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    Audio = 8,
    Video = 9,
    ScriptData = 16,
    Other = 31
}

impl From<u8> for TagType {
    fn from(tag_type: u8) -> Self {
        use TagType::*;

        match tag_type {
            8 => Audio,
            9 => Video,
            16 => ScriptData,
            _ => Other
        }
    }
}

impl From<TagType> for u8 {
    fn from(tag_type: TagType) -> Self {
        tag_type as u8
    }
}

/// The FLV body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlvTag {
    timestamp: Duration,
    inner_tag: InnerTag
}

impl FlvTag {
    const MESSAGE_ID: u32 = 0;

    /// Constructs a FlvTag.
    pub fn new(timestamp: Duration, inner_tag: InnerTag) -> Self {
        Self { timestamp, inner_tag }
    }

    /// Gets the type of this tag.
    /// See [`TagType`] for detail what's returned.
    ///
    /// [`TagType`]: TagType
    pub fn get_tag_type(&self) -> TagType {
        match self.inner_tag {
            InnerTag::Audio(_) => TagType::Audio,
            InnerTag::Video(_) => TagType::Video,
            InnerTag::ScriptData(_) => TagType::ScriptData
        }
    }

    /// Gets the timestamp that created this tag.
    pub fn get_timestamp(&self) -> Duration {
        self.timestamp
    }

    pub fn get_inner_tag(&self) -> &InnerTag {
        &self.inner_tag
    }
}

impl Decoder<FlvTag> for ByteBuffer {
    fn decode(&mut self) -> IOResult<FlvTag> {
        let tag_type = self.get_u8()?;
        // NOTE: This is a data size not to be used.
        self.get_u24_be()?;
        let mut timestamp = self.get_u24_be()? as u32;
        let timestamp_extended = self.get_u8()? as u32;
        // NOTE: This is the message ID that is always 0.
        self.get_u24_be()?;
        let inner_tag: InnerTag = match TagType::from(tag_type) {
            TagType::Audio => Decoder::<AudioTag>::decode(self).map(InnerTag::Audio)?,
            TagType::Video => Decoder::<VideoTag>::decode(self).map(InnerTag::Video)?,
            TagType::ScriptData => Decoder::<ScriptDataTag>::decode(self).map(InnerTag::ScriptData)?,
            _ => return Err(unknown_tag(tag_type))
        };

        timestamp |= timestamp_extended << 23;
        Ok(FlvTag::new(Duration::from_millis(timestamp as u64), inner_tag))
    }
}

impl Encoder<FlvTag> for ByteBuffer {
    fn encode(&mut self, flv_tag: &FlvTag) {
        let timestamp = flv_tag.get_timestamp().as_millis() as u32;
        let data: Vec<u8> = match flv_tag.get_inner_tag() {
            InnerTag::Audio(ref audio_tag) => {
                let mut buffer = ByteBuffer::default();
                buffer.encode(audio_tag);
                buffer.into()
            },
            InnerTag::Video(ref video_tag) => {
                let mut buffer = ByteBuffer::default();
                buffer.encode(video_tag);
                buffer.into()
            },
            InnerTag::ScriptData(ref script_data_tag) => {
                let mut buffer = ByteBuffer::default();
                buffer.encode(script_data_tag);
                buffer.into()
            }
        };

        self.put_u8(flv_tag.get_tag_type().into());
        self.put_u24_be(data.len() as u32);
        self.put_u24_be(timestamp & 0x00FFFFFF);
        self.put_u8((timestamp >> 23) as u8);
        // NOTE: This is the message ID that is always 0.
        self.put_u24_be(0);
        self.put_bytes(&data);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ecma_array,
        messages::amf::v0::Number
    };
    use super::*;

    #[test]
    fn decode_flv_tag() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(
            &ScriptDataTag::new(
                "onMetaData".into(),
                ecma_array!(
                    "audiocodecid" => Number::from(0),
                    "videocodecid" => Number::from(2)
                )
            )
        );
        let data: Vec<u8> = buffer.into();

        let mut buffer = ByteBuffer::default();
        buffer.put_u8(TagType::ScriptData.into());
        buffer.put_u24_be(data.len() as u32);
        // NOTE: This is the timestamp in a flv tag which is at the head position.
        buffer.put_u32_be(0);
        buffer.put_u24_be(0);
        buffer.put_bytes(&data);
        assert!(Decoder::<FlvTag>::decode(&mut buffer).is_ok());

        let mut buffer = ByteBuffer::default();
        buffer.put_u8(TagType::Other.into());
        buffer.put_u24_be(data.len() as u32);
        buffer.put_u32_be(0);
        buffer.put_u24_be(0);
        buffer.put_bytes(&data);
        assert!(Decoder::<FlvTag>::decode(&mut buffer).is_err())
    }

    #[test]
    fn encode_flv_tag() {
        let expected_data = ScriptDataTag::new(
            "onMetaData".into(),
            ecma_array!(
                "audiocodecid" => Number::from(0),
                "videocodecid" => Number::from(2)
            )
        );
        let mut buffer = ByteBuffer::default();
        buffer.encode(&FlvTag::new(Duration::default(), InnerTag::ScriptData(expected_data.clone())));
        let tag_type: TagType = buffer.get_u8().unwrap().into();
        let data_size = buffer.get_u24_be().unwrap();
        let timestamp = buffer.get_u32_be().unwrap();
        let message_id = buffer.get_u24_be().unwrap();
        let remained = buffer.remained() as u32;
        let actual_data: ScriptDataTag = buffer.decode().unwrap();
        assert_eq!(TagType::ScriptData, tag_type);
        assert_eq!(data_size, remained);
        assert_eq!(0, timestamp);
        assert_eq!(0, message_id);
        assert_eq!(expected_data, actual_data)
    }
}
