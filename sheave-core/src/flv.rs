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
//!    * Previous Tag Size (32 bits. this of the first is 0)
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
mod not_flv_container;
mod unknown_tag;
pub mod tags;

use std::{
    fmt::{
        Display,
        Formatter,
        Result as FormatResult
    },
    fs::OpenOptions,
    io::{
        Read,
        Result as IOResult,
        Seek,
        SeekFrom,
        Write
    },
    path::Path,
    time::Duration
};
use super::{
    ByteBuffer,
    Decoder
};
use self::tags::*;
pub use self::{
    not_flv_container::*,
    unknown_tag::*,
};

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
/// This holds just 2 elements:
///
/// * A path to actual FLV file
/// * Offset in FLV file (for reading).
///
/// By not to hold actual file handle, this makes plural users to read/write FLV file not to bump.
/// Actual file handle is gotten only while file opens/creates and file reads/writes.
#[derive(Debug, Clone)]
pub struct Flv {
    offset: u64,
    path: String
}

impl Flv {
    const SIGNATURE: &'static str = "FLV";
    const LATEST_VERSION: u8 = 10;
    const HEADER_LEN: usize = 9;

    /// Constructs a FLV container from a file.
    ///
    /// # Errors
    ///
    /// When passed file isn't the FLV container:
    ///
    /// * It doesn't start with "FLV".
    /// * It doesn't have the FLV header (requires 9 bytes).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::{
    ///     fs::{
    ///         File,
    ///         OpenOptions
    ///     },
    ///     io::{
    ///         Read,
    ///         Seek,
    ///         SeekFrom,
    ///         Write
    ///     }
    /// };
    /// use sheave_core::flv::*;
    ///
    /// // When the input length less than 13.
    /// let mut input = OpenOptions::new()
    ///     .write(true)
    ///     .create(true)
    ///     .truncate(true)
    ///     .open("/tmp/err1.flv").unwrap();
    /// let result = Flv::open("/tmp/err1.flv");
    /// assert!(result.is_err());
    ///
    /// // When the signature isn't "FLV".
    /// let mut input = OpenOptions::new()
    ///     .write(true)
    ///     .create(true)
    ///     .truncate(true)
    ///     .open("/tmp/err2.flv").unwrap();
    /// input.write("F4V".as_bytes()).unwrap();
    /// input.flush().unwrap();
    /// input.seek(SeekFrom::Start(0)).unwrap();
    /// let result = Flv::open("/tmp/err2.flv");
    /// assert!(result.is_err());
    ///
    /// // Ok.
    /// let mut input = OpenOptions::new()
    ///     .write(true)
    ///     .create(true)
    ///     .truncate(true)
    ///     .open("/tmp/ok.flv").unwrap();
    /// let mut bytes: [u8; 9] = [0; 9];
    /// bytes[..3].copy_from_slice("FLV".as_bytes());
    /// input.write(&bytes).unwrap();
    /// // NOTE: This is a previous tag size at the head position.
    /// input.write(&0u32.to_be_bytes()).unwrap();
    /// input.flush().unwrap();
    /// input.seek(SeekFrom::Start(0)).unwrap();
    /// let result = Flv::open("/tmp/ok.flv");
    /// assert!(result.is_ok())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> IOResult<Self> {
        let referred_path = path.as_ref();
        let mut file = OpenOptions::new()
            .read(true)
            .open(referred_path)?;
        let mut flv_header: [u8; Self::HEADER_LEN] = [0; Self::HEADER_LEN];
        file.read(&mut flv_header)?;

        let signature = &flv_header[..3];

        if signature != Self::SIGNATURE.as_bytes() {
            Err(not_flv_container(&flv_header[..3]))
        } else {
            Ok(
                Self {
                    // NOTE: Seeks to the position of first FLV tag.
                    offset: 13,
                    path: referred_path.to_str().unwrap().into(),
                }
            )
        }
    }

    /// Constructs an empty FLV container from a name.
    pub fn create<P: AsRef<Path>>(path: P) -> IOResult<Self> {
        let referred_path = path.as_ref();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(referred_path)?;
        let mut flv_header: [u8; Self::HEADER_LEN] = [0; Self::HEADER_LEN];
        flv_header[..3].copy_from_slice(Self::SIGNATURE.as_bytes());
        flv_header[3] = Self::LATEST_VERSION;
        flv_header[8] = Self::HEADER_LEN as u8;
        file.write(&flv_header)?;
        file.write(&0u32.to_be_bytes())?;
        file.flush()?;
        Ok(
            Self {
                // NOTE: Seeks to the position of first FLV tag.
                offset: 13,
                path: referred_path.to_str().unwrap().into(),
            }
        )
    }

    /// Gets the current FLV version.
    pub fn get_version(&self) -> IOResult<u8> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.path)?;
        file.seek(SeekFrom::Start(3))?;
        let mut version_byte: [u8; 1] = [0; 1];
        file.read(&mut version_byte)?;
        Ok(u8::from_be_bytes(version_byte))
    }

    fn set_flags(&self, flags: u8) -> IOResult<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.path)?;
        file.seek(SeekFrom::Start(4))?;
        file.write(&flags.to_be_bytes())?;
        file.flush()
    }

    /// Checks whether FLV container has audio data.
    pub fn has_audio(&self) -> IOResult<bool> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.path)?;
        file.seek(SeekFrom::Start(4))?;
        let mut flags_byte: [u8; 1] = [0; 1];
        file.read(&mut flags_byte)?;
        Ok((flags_byte[0] & 0x04) != 0)
    }

    /// Checks whether FLV container has video data.
    pub fn has_video(&self) -> IOResult<bool> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.path)?;
        file.seek(SeekFrom::Start(4))?;
        let mut flags_byte: [u8; 1] = [0; 1];
        file.read(&mut flags_byte)?;
        Ok((flags_byte[0] & 0x01) != 0)
    }

    /// Appends a FLV tag into the tag container.
    ///
    /// This reuses the Codec IDs in the metadata for checking whether FLV has audio/video data.
    ///
    /// That is,
    ///
    /// * If `audiocodecid` exists, FLV contains auduo data.
    /// * Or if `videocodecid` exists, FLV contains video data.
    /// * Otherwise FLV consists of just script data.
    pub fn append_flv_tag(&self, flv_tag: FlvTag) -> IOResult<()> {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.path)?;

        if let TagType::ScriptData = flv_tag.get_tag_type() {
            let mut buffer: ByteBuffer = flv_tag.get_data().to_vec().into();
            let script_data: ScriptDataTag = buffer.decode()?;
            let has_audio = script_data.get_value().get_properties().get("audiocodecid").is_some() as u8;
            let has_video = script_data.get_value().get_properties().get("videocodecid").is_some() as u8;
            self.set_flags((has_audio << 2) | has_video)?;
        }

        let timestamp_bytes = (flv_tag.get_timestamp().as_millis() as u32).to_be_bytes();
        let data_size = flv_tag.get_data().len();
        let mut metadata: [u8; METADATA_LEN] = [0; METADATA_LEN];
        metadata[0] = flv_tag.get_tag_type().into();
        metadata[1..4].copy_from_slice(&data_size.to_be_bytes()[5..]);
        metadata[4..7].copy_from_slice(&timestamp_bytes[1..]);
        metadata[7] = timestamp_bytes[0];
        // NOTE: This is the message ID that is currently always 0.
        metadata[8..].copy_from_slice(&DEFAULT_MESSAGE_ID.to_be_bytes()[..3]);

        file.write(&metadata)?;
        file.write(flv_tag.get_data())?;
        file.write(&(METADATA_LEN + data_size).to_be_bytes()[4..])?;
        file.flush()
    }
}

impl Iterator for Flv {
    type Item = IOResult<FlvTag>;

    /// Reads a FLV tag from the path.
    ///
    /// Note this can return some error when following causes:
    ///
    /// * `UnknownTag`
    ///
    /// When any undefined tag type found.
    /// Currently, the tag type should be one of 8(Audio), 9(Video) or 18(Data) in the FLV container.
    /// That is, this library doesn't know any way of handling other type.
    ///
    /// * Something else
    ///
    /// When reading/seeking got failed by some cause.
    fn next(&mut self) -> Option<Self::Item> {
        let mut file = match OpenOptions::new().read(true).open(&self.path) {
            Ok(file) => file,
            Err(e) => return Some(Err(e))
        };

        if let Err(e) = file.seek(SeekFrom::Start(self.offset)) {
            return Some(Err(e))
        }

        let mut metadata_bytes: [u8; METADATA_LEN] = [0; METADATA_LEN];
        match file.read(&mut metadata_bytes) {
            Err(e) => return Some(Err(e)),
            Ok(0) => return None,
            _ => {}
        }

        let tag_type_byte = metadata_bytes[0] & 0x1f;
        let tag_type: TagType = match tag_type_byte {
            8 | 9 | 18 => tag_type_byte.into(),
            other => return Some(Err(unknown_tag(other)))
        };

        let mut data_size_bytes: [u8; 4] = [0; 4];
        data_size_bytes[1..].copy_from_slice(&metadata_bytes[1..4]);
        let data_size = u32::from_be_bytes(data_size_bytes);
        let mut data: Vec<u8> = Vec::with_capacity(data_size as usize);
        unsafe { data.set_len(data_size as usize); }
        if let Err(e) = file.read(&mut data) {
            return Some(Err(e))
        }

        // NOTE: Previous Tag Size is unnecessary in reading.
        if let Err(e) = file.seek(SeekFrom::Current(4)) {
            return Some(Err(e))
        }

        let mut timestamp_bytes: [u8; 4] = [0; 4];
        timestamp_bytes[1..].copy_from_slice(&metadata_bytes[4..7]);
        let timestamp = u32::from_be_bytes(timestamp_bytes) | ((metadata_bytes[8] as u32) << 23);


        self.offset = match file.stream_position() {
            Err(e) => return Some(Err(e)),
            Ok(offset) => offset
        };
        Some(Ok(FlvTag::new(tag_type, Duration::from_millis(timestamp as u64), data)))
    }
}
