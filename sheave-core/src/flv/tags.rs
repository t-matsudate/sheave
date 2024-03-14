mod audio;
mod video;
mod script_data;

use std::time::Duration;
use super::EncryptionHeader;
pub use self::{
    audio::*,
    video::*,
    script_data::*
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InnerTag {
    Audio(AudioTag),
    Video(VideoTag),
    ScriptData(ScriptDataTag)
}

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

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlvTag {
    timestamp: Duration,
    encryption_header: Option<EncryptionHeader>,
    inner_tag: InnerTag
}

impl FlvTag {
    const MESSAGE_ID: u32 = 0;

    pub fn new(timestamp: Duration, encryption_header: Option<EncryptionHeader>, inner_tag: InnerTag) -> Self {
        Self { timestamp, encryption_header, inner_tag }
    }

    pub fn is_filtered(&self) -> bool {
        self.encryption_header.is_some()
    }

    pub fn get_tag_type(&self) -> TagType {
        match self.inner_tag {
            InnerTag::Audio(_) => TagType::Audio,
            InnerTag::Video(_) => TagType::Video,
            InnerTag::ScriptData(_) => TagType::ScriptData
        }
    }

    pub fn get_timestamp(&self) -> Duration {
        self.timestamp
    }
}
