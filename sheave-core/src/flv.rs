mod encryption_header;
mod tags;

use std::{
    fmt::{
        Display,
        Formatter,
        Result as FormatResult
    },
    time::{
        Duration,
        Instant
    }
};
pub use self::{
    encryption_header::*,
    tags::*
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlvHeader {
    has_audio: bool,
    has_video: bool,
    version: u8
}

impl FlvHeader {
    const SIGNATURE: &'static str = "FLV";
    const LATEST_VERSION: u8 = 10;
    const LEN: u32 = 9;
}

impl Default for FlvHeader {
    fn default() -> Self {
        Self {
            has_audio: false,
            has_video: false,
            version: Self::LATEST_VERSION
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterName {
    Encryption,
    SelectiveEncryption
}

impl Display for FilterName {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        use FilterName::*;

        match *self {
            Encryption => write!(f, "Encryption"),
            SelectiveEncryption => write!(f, "SE")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flv {
    created_at: Instant,
    header: FlvHeader,
    filter_name: Option<FilterName>,
    body: Vec<FlvTag>
}

impl Flv {
    fn compute_timestamp(&self) -> Duration {
        if self.body.is_empty() {
            Duration::default()
        } else {
            self.created_at.elapsed()
        }
    }

    pub fn get_version(&self) -> u8 {
        self.header.version
    }

    pub fn has_audio(&self) -> bool {
        self.header.has_audio
    }

    pub fn has_video(&self) -> bool {
        self.header.has_video
    }

    fn append_meta_data(&mut self, meta_data: ScriptDataTag) {
        let timestamp = self.compute_timestamp();

        self.header.has_audio = meta_data.get_value().get_properties().get("audiocodecid").is_some();
        self.header.has_video = meta_data.get_value().get_properties().get("videocodecid").is_some();
        self.body.push(FlvTag::new(timestamp, None, InnerTag::ScriptData(meta_data)));
    }

    fn append_audio(&mut self, audio: AudioTag) {
        let timestamp = self.compute_timestamp();

        self.body.push(FlvTag::new(timestamp, None, InnerTag::Audio(audio)));
    }

    fn append_video(&mut self, video: VideoTag) {
        let timestamp = self.compute_timestamp();

        self.body.push(FlvTag::new(timestamp, None, InnerTag::Video(video)));
    }

    pub fn append_flv_tag(&mut self, flv_tag: InnerTag) {
        match flv_tag {
            InnerTag::Audio(audio_tag) => self.append_audio(audio_tag),
            InnerTag::Video(video_tag) => self.append_video(video_tag),
            InnerTag::ScriptData(script_data_tag) => self.append_meta_data(script_data_tag)
        }
    }

    pub fn get_current_bodies(&self) -> &[FlvTag] {
        &self.body
    }
}

impl Default for Flv {
    fn default() -> Self {
        Self {
            created_at: Instant::now(),
            header: FlvHeader::default(),
            filter_name: Option::default(),
            body: Vec::default(),
        }
    }
}
