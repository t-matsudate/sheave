//! # The FLV File Format
//!
//! In RTMP, Both of the client and the server send/receive actual multi media data as the FLV file format.
//! Its format consists of:
//!
//! 1. FLV header
//!    * Signature ("FLV")
//!    * Version (8 bits)
//!    * Reserved (5 bits)
//!    * Whether some audio data is contained (1 bit)
//!    * Reserved (1 bit)
//!    * Whether some video data is contained (1 bit)
//!    * Offset to FLV data (that is, a size of this header = 9) (32 bits)
//! 2. FLV file body
//!    * PreviousTagSize (32 bits. this of the first is 0)
//!    * FLV Tag (arbitrary size)
//!
//! Note the FLV header is skipped by almost RTMP tools.
//!
//! ## FLV Tag
//!
//! FLV Tag is a part of actual FLV bodies.
//! FLV Tag consists of:
//!
//! * [`AudioTag`]
//! * [`VideoTag`]
//! * [`ScriptDataTag`]
//!
//! [`AudioTag`]: tags::AudioTag
//! [`VideoTag`]: tags::VideoTag
//! [`ScriptDataTag`]: tags::ScriptDataTag
mod encryption_header;
pub mod tags;

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
use self::tags::*;
pub use self::encryption_header::*;

/// An outermost header part of the FLV.
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

/// Patterns of the FilterName field.
/// Currently, FilterName consists of:
///
/// * `"Encryption"`
/// * `"SE"` (Selective Encryption)
///
/// But these are strings so we will be hard to refuse other values at this rate.
/// Therefore this limits any FilterName pattern to fix it to an enum.
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

/// The FLV container.
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

    /// Gets the current FLV version.
    pub fn get_version(&self) -> u8 {
        self.header.version
    }

    /// Checks whether this contains audio data.
    pub fn has_audio(&self) -> bool {
        self.header.has_audio
    }

    /// Checks whether this contains video data.
    pub fn has_video(&self) -> bool {
        self.header.has_video
    }

    /// Appends FLV metadata into the tag container.
    /// This library reuses the Codec IDs in the metadata for checking whether FLV has audio/video data.
    /// That is,
    ///
    /// If `audiocodecid` exists, FLV contains auduo data.
    /// Or if `videocodecid` exists, FLV contains video data.
    /// Otherwise FLV consists of just script data.
    pub fn append_meta_data(&mut self, meta_data: ScriptDataTag) {
        let timestamp = self.compute_timestamp();

        self.header.has_audio = meta_data.get_value().get_properties().get("audiocodecid").is_some();
        self.header.has_video = meta_data.get_value().get_properties().get("videocodecid").is_some();
        self.body.push(FlvTag::new(timestamp, None, InnerTag::ScriptData(meta_data)));
    }

    /// Appends audio data into the tag container.
    pub fn append_audio(&mut self, audio: AudioTag) {
        let timestamp = self.compute_timestamp();

        self.body.push(FlvTag::new(timestamp, None, InnerTag::Audio(audio)));
    }

    /// Appends video data into the tag container.
    pub fn append_video(&mut self, video: VideoTag) {
        let timestamp = self.compute_timestamp();

        self.body.push(FlvTag::new(timestamp, None, InnerTag::Video(video)));
    }

    /// Gets current body data as the slice.
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
