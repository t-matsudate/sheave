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
mod not_flv_container;
mod unknown_tag;
mod encryption_header;
pub mod tags;

use std::{
    fmt::{
        Display,
        Formatter,
        Result as FormatResult
    },
    io::{
        Result as IOResult,
        SeekFrom
    },
    pin::Pin,
    task::{
        Context as FutureContext,
        Poll
    },
    time::{
        Duration,
        Instant
    }
};
use futures::{
    Stream,
    ready
};
use tokio::{
    fs::File,
    io::{
        AsyncRead,
        AsyncReadExt,
        AsyncSeek,
        AsyncSeekExt,
        AsyncWriteExt,
        ReadBuf
    }
};
use super::{
    ByteBuffer,
    Decoder,
    Encoder
};
pub use self::{
    not_flv_container::*,
    unknown_tag::*,
    encryption_header::*,
    tags::*
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
#[derive(Debug)]
pub struct Flv {
    version: u8,
    has_audio: bool,
    has_video: bool,
    body: File
}

impl Flv {
    pub const SIGNATURE: &'static str = "FLV";
    pub const LATEST_VERSION: u8 = 10;
    pub const LEN: usize = 9;

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
    /// use std::io::SeekFrom;
    /// use tokio::{
    ///     fs::{
    ///         File,
    ///         OpenOptions
    ///     },
    ///     io::{
    ///         AsyncSeekExt,
    ///         AsyncWriteExt
    ///     }
    /// };
    /// use sheave_core::flv::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     // When the input length less than 13.
    ///     let mut input = OpenOptions::new()
    ///         .read(true)
    ///         .write(true)
    ///         .create(true)
    ///         .open("/tmp/err1.flv").await.unwrap();
    ///     let result = Flv::create_from_file(input).await;
    ///     assert!(result.is_err());
    ///
    ///     // When the signature isn't "FLV".
    ///     let mut input = OpenOptions::new()
    ///         .read(true)
    ///         .write(true)
    ///         .create(true)
    ///         .open("/tmp/err2.flv").await.unwrap();
    ///     input.write("F4V".as_bytes()).await.unwrap();
    ///     input.flush().await.unwrap();
    ///     input.seek(SeekFrom::Start(0)).await.unwrap();
    ///     let result = Flv::create_from_file(input).await;
    ///     assert!(result.is_err());
    ///
    ///     // Ok.
    ///     let mut input = OpenOptions::new()
    ///         .read(true)
    ///         .write(true)
    ///         .create(true)
    ///         .open("/tmp/ok.flv").await.unwrap();
    ///     let mut bytes: [u8; 13] = [0; 13];
    ///     bytes[0] = b'F';
    ///     bytes[1] = b'L';
    ///     bytes[2] = b'V';
    ///     input.write(&bytes).await.unwrap();
    ///     input.flush().await.unwrap();
    ///     input.seek(SeekFrom::Start(0)).await.unwrap();
    ///     let result = Flv::create_from_file(input).await;
    ///     assert!(result.is_ok())
    /// }
    /// ```
    pub async fn create_from_file(mut file: File) -> IOResult<Self> {
        let mut flv_header: [u8; Self::LEN] = [0; Self::LEN];
        file.read(&mut flv_header).await?;

        let signature = &flv_header[..3];

        if signature != Flv::SIGNATURE.as_bytes() {
            Err(not_flv_container(&flv_header[..3]))
        } else {
            // NOTE: This is a previous tag size at the head position.
            file.seek(SeekFrom::Current(4)).await?;

            let version = flv_header[3];
            let flags = flv_header[4];
            let has_audio = (flags & 0x03) != 0;
            let has_video = (flags & 0x01) != 0;
            Ok(
                Self {
                    version,
                    has_audio,
                    has_video,
                    body: file
                }
            )
        }
    }

    /// Constructs an empty FLV container from a name.
    /// This imprints an empty header byte at creating its container.
    /// Because its header is written before shutting down streams.
    pub async fn create_from_name(name: &str) -> IOResult<Self> {
        let body = File::create(name).await?;
        body.write([0u8; 13].as_slice()).await?;
        body.flush().await?;

        Ok(
            Self {
                version: Self::LATEST_VERSION,
                has_audio: false,
                has_video: false,
                body
            }
        )
    }

    async fn compute_timestamp(&self) -> Duration {
        let metadata = self.body.metadata().await.unwrap();

        // NOTE: 13 is the length of FLV header and a length of previous tag size following it.
        // We can ignore these by following reasons:
        //
        // * We send/receive only FLV tags while communication.
        // * the tag size at the head position indicates nothing.
        if metadata.len() > 13 {
            Instant::now().elapsed()
        } else {
            Duration::default()
        }
    }

    /// Gets the current FLV version.
    pub fn get_version(&self) -> u8 {
        self.version
    }

    /// Checks whether this contains audio data.
    pub fn has_audio(&self) -> bool {
        self.has_audio
    }

    /// Checks whether this contains video data.
    pub fn has_video(&self) -> bool {
        self.has_video
    }

    /// Appends a FLV tag into the tag container.
    /// This reuses the Codec IDs in the metadata for checking whether FLV has audio/video data.
    /// That is,
    ///
    /// If `audiocodecid` exists, FLV contains auduo data.
    /// Or if `videocodecid` exists, FLV contains video data.
    /// Otherwise FLV consists of just script data.
    pub async fn append_flv_tag(&mut self, flv_tag: FlvTag) -> IOResult<()> {
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
                self.has_audio = script_data_tag.get_value().get_properties().get("audiocodecid").is_some();
                self.has_video = script_data_tag.get_value().get_properties().get("videocodecid").is_some();

                let mut buffer = ByteBuffer::default();
                buffer.encode(script_data_tag);
                buffer.into()
            }
        };
        let timestamp = (self.compute_timestamp().await.as_millis() as u32).to_be_bytes();

        self.body.write_u8(flv_tag.get_tag_type().into()).await?;
        self.body.write(&(data.len() as u32).to_be_bytes()[1..]).await?;
        self.body.write(&timestamp[1..]).await?;
        self.body.write_u8(timestamp[0]).await?;
        // NOTE: This is the message ID that is always 0.
        self.body.write(&0u32.to_be_bytes()[1..]).await?;
        self.body.write(&data).await?;
        self.body.write_u32(11 + data.len() as u32).await?;
        self.body.flush().await
    }

    /// Shuts down writing stream for this container.
    /// Note that FLV header fields are written in this moment.
    pub async fn shutdown(&mut self) -> IOResult<()> {
        self.body.rewind().await?;

        self.body.write(Self::SIGNATURE.as_bytes()).await?;
        let version = self.version;
        self.body.write_u8(version).await?;
        let has_audio = self.has_audio as u8;
        let has_video = self.has_video as u8;
        self.body.write_u8((has_audio << 2) | has_video).await?;
        self.body.write_u32(9).await?;
        self.body.write_u32(0).await?;

        self.body.shutdown().await
    }
}

impl Stream for Flv {
    type Item = IOResult<FlvTag>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Option<Self::Item>> {
        let mut body = Pin::new(&mut self.body);
        let mut metadata_bytes: [u8; 11] = [0; 11];
        let mut buf = ReadBuf::new(&mut metadata_bytes);

        if let Err(e) = ready!(body.as_mut().poll_read(cx, &mut buf)) {
            return Poll::Ready(Some(Err(e)))
        } else if buf.filled().len() == 0 {
            return Poll::Ready(None)
        }

        let mut data_size_bytes: [u8; 4] = [0; 4];
        data_size_bytes[1..].copy_from_slice(&metadata_bytes[1..4]);
        let data_size = u32::from_be_bytes(data_size_bytes);
        let mut data_bytes: Vec<u8> = Vec::with_capacity(data_size as usize);
        unsafe { data_bytes.set_len(data_size as usize); }
        let mut buf = ReadBuf::new(&mut data_bytes);

        if let Err(e) = ready!(body.as_mut().poll_read(cx, &mut buf)) {
            return Poll::Ready(Some(Err(e)))
        }

        // NOTE: Previous Tag Size is unnecessary in reading.
        body.as_mut().start_seek(SeekFrom::Current(4))?;

        let tag_type_byte = metadata_bytes[0] & 0x1f;
        let mut timestamp_bytes: [u8; 4] = [0; 4];
        timestamp_bytes[1..].copy_from_slice(&metadata_bytes[4..7]);
        let timestamp = u32::from_be_bytes(timestamp_bytes) | ((metadata_bytes[8] as u32) << 23);

        let mut buffer: ByteBuffer = data_bytes.into();
        let inner_tag = match TagType::from(tag_type_byte) {
            TagType::Audio => match Decoder::<AudioTag>::decode(&mut buffer) {
                Ok(audio_tag) => InnerTag::Audio(audio_tag),
                Err(e) => return Poll::Ready(Some(Err(e)))
            },
            TagType::Video => match Decoder::<VideoTag>::decode(&mut buffer) {
                Ok(video_tag) => InnerTag::Video(video_tag),
                Err(e) => return Poll::Ready(Some(Err(e)))
            },
            TagType::ScriptData => match Decoder::<ScriptDataTag>::decode(&mut buffer) {
                Ok(script_data_tag) => InnerTag::ScriptData(script_data_tag),
                Err(e) => return Poll::Ready(Some(Err(e)))
            },
            _ => return Poll::Ready(Some(Err(unknown_tag(tag_type_byte))))
        };

        Poll::Ready(Some(Ok(FlvTag::new(Duration::from_millis(timestamp as u64), inner_tag))))
    }
}
