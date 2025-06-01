use std::io::{
    Error as IOError,
    Result as IOResult
};
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::Audio
};

/// Representation of the SoundFormat field of the AudioTag.
///
/// Variants correspond to respectively following numbers:
///
/// |Pattern|Number|
/// | :- | :- |
/// |`LinearPcmNe`|`0`|
/// |`AdPcm`|`1`|
/// |`Mp3`|`2`|
/// |`LinearPcmLe`|`3`|
/// |`NellyMoserMono16`|`4`|
/// |`NellyMoserMono8`|`5`|
/// |`NellyMoserStereo`|`6`|
/// |`G711A`|`7`|
/// |`G711Mu`|`8`|
/// |`Reserved`|`9`|
/// |`Aac`|`10`|
/// |`Speex`|`11`|
/// |`Mp3_8`|`14`|
/// |`DeviceSpecific`|`15`|
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundFormat {
    LinearPcmNe,
    AdPcm,
    Mp3,
    LinearPcmLe,
    NellyMoserMono16,
    NellyMoserMono8,
    NellyMoserStereo,
    G711A,
    G711Mu,
    Reserved,
    Aac,
    Speex,
    Mp3_8 = 14,
    DeviceSpecific
}

impl SoundFormat {
    pub fn is_aac(&self) -> bool {
        match *self {
            SoundFormat::Aac => true,
            _ => false
        }
    }
}

impl From<u8> for SoundFormat {
    /// Converts a SoundFormat field into a variant.
    ///
    /// # Panics
    ///
    /// Because of FLV specification, this is implemented in such a way as to emit a panic when is passed a value either 12, 13 or any of above 15.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::panic::catch_unwind;
    /// use sheave_core::flv::tags::{
    ///     SoundFormat,
    ///     SoundFormat::*
    /// };
    ///
    /// assert_eq!(Speex, SoundFormat::from(11)); // => ok
    /// assert_eq!(Mp3_8, SoundFormat::from(14)); // => ok
    /// assert_eq!(DeviceSpecific, SoundFormat::from(15)); // => ok
    /// assert!(catch_unwind(|| SoundFormat::from(12)).is_err()); // => this will be backtrace.
    /// assert!(catch_unwind(|| SoundFormat::from(13)).is_err()); // => this is too.
    /// assert!(catch_unwind(|| SoundFormat::from(16)).is_err()) // => same as above.
    /// ```
    fn from(sound_format: u8) -> Self {
        use SoundFormat::*;

        match sound_format {
            0 => LinearPcmNe,
            1 => AdPcm,
            2 => Mp3,
            3 => LinearPcmLe,
            4 => NellyMoserMono16,
            5 => NellyMoserMono8,
            6 => NellyMoserStereo,
            7 => G711A,
            8 => G711Mu,
            9 => Reserved,
            10 => Aac,
            11 => Speex,
            14 => Mp3_8,
            15 => DeviceSpecific,
            _ => unreachable!("SoundFormat")
        }
    }
}

impl From<SoundFormat> for u8 {
    fn from(sound_format: SoundFormat) -> Self {
        sound_format as u8
    }
}

/// Representation of the SoundRate field.
///
/// Variants correspond to respectively following numbers:
///
/// |Variant|Number|
/// | :- | :- |
/// |`FivePointFive`|`0`|
/// |`Eleven`|`1`|
/// |`TwentyTwo`|`2`|
/// |`FourtyFour`|`3`|
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundRate {
    FivePointFive,
    Eleven,
    TwentyTwo,
    FourtyFour
}

impl From<u8> for SoundRate {
    /// Converts a SoundRate field into a variant.
    ///
    /// # Panics
    ///
    /// Because of FLV specification, this is implemented in such a way as to emit a panic when is passed any value above 3.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::panic::catch_unwind;
    /// use sheave_core::flv::tags::{
    ///     SoundRate,
    ///     SoundRate::*
    /// };
    ///
    /// assert_eq!(FivePointFive, SoundRate::from(0)); // => ok
    /// assert_eq!(Eleven, SoundRate::from(1)); // => ok
    /// assert_eq!(TwentyTwo, SoundRate::from(2)); // => ok
    /// assert_eq!(FourtyFour, SoundRate::from(3)); // => ok
    /// assert!(catch_unwind(|| SoundRate::from(4)).is_err()) // => this will be backtrace.
    /// ```
    fn from(sound_rate: u8) -> Self {
        use SoundRate::*;

        match sound_rate {
            0 => FivePointFive,
            1 => Eleven,
            2 => TwentyTwo,
            3 => FourtyFour,
            _ => unreachable!("SoundRate.")
        }
    }
}

impl From<SoundRate> for u8 {
    fn from(sound_rate: SoundRate) -> Self {
        sound_rate as u8
    }
}

/// The header of the AudioTag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioTagHeader {
    sound_format: SoundFormat,
    sound_rate: SoundRate,
    is_sixteen_bits: bool,
    is_stereo: bool,
    is_aac_raw: Option<bool>
}

impl AudioTagHeader {
    /// Consturcts a AudioTagHeader.
    pub fn new(sound_format: SoundFormat, sound_rate: SoundRate, is_sixteen_bits: bool, is_stereo: bool, is_aac_raw: Option<bool>) -> Self {
        Self { sound_format, sound_rate, is_sixteen_bits, is_stereo, is_aac_raw }
    }
}

impl Decoder<AudioTagHeader> for ByteBuffer {
    /// Decodes bytes into a AudioTagHeader.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When some field misses.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     flv::tags::AudioTagHeader
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(0);
    /// assert!(Decoder::<AudioTagHeader>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<AudioTagHeader>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    fn decode(&mut self) -> IOResult<AudioTagHeader> {
        let byte = self.get_u8()?;
        let sound_format: SoundFormat = ((byte & 0xf0) >> 4).into();
        let sound_rate: SoundRate = ((byte & 0x0c) >> 2).into();
        let is_sixteen_bits = ((byte & 0x02) >> 1) == 1;
        let is_stereo = (byte & 0x01) == 1;

        let is_aac_raw = if !sound_format.is_aac() {
            None
        } else {
            let byte = self.get_u8()?;
            Some(byte == 1)
        };

        Ok(AudioTagHeader { sound_format, sound_rate, is_sixteen_bits, is_stereo, is_aac_raw })
    }
}

impl Encoder<AudioTagHeader> for ByteBuffer {
    /// Encodes a AudioTagHeader into bytes.
    fn encode(&mut self, audio_tag_header: &AudioTagHeader) {
        let mut byte = u8::from(audio_tag_header.sound_format) << 4;
        byte |= u8::from(audio_tag_header.sound_rate) << 2;
        byte |= u8::from(audio_tag_header.is_sixteen_bits) << 1;
        byte |= u8::from(audio_tag_header.is_stereo);
        self.put_u8(byte);

        if let Some(b) = audio_tag_header.is_aac_raw {
            self.put_u8(u8::from(b));
        }
    }
}

/// The audio data format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioTag {
    header: AudioTagHeader,
    body: Vec<u8>
}

impl AudioTag {
    /// Constructs a AudioTag.
    pub fn new(header: AudioTagHeader, body: Vec<u8>) -> Self {
        Self { header, body }
    }
}

impl Decoder<AudioTag> for ByteBuffer {
    /// Decodes bytes into a AudioTag.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When some field misses.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::fill;
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     flv::tags::{
    ///         AudioTag,
    ///         AudioTagHeader,
    ///         SoundFormat,
    ///         SoundRate
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AudioTagHeader::new(SoundFormat::LinearPcmNe, SoundRate::FivePointFive, false, false, None));
    /// let mut bytes: [u8; 127] = [0; 127];
    /// fill(&mut bytes);
    /// buffer.put_bytes(&bytes);
    /// assert!(Decoder::<AudioTag>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<AudioTag>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    fn decode(&mut self) -> IOResult<AudioTag> {
        let header: AudioTagHeader = self.decode()?;
        let remained = self.remained();
        let body = self.get_bytes(remained)?.to_vec();

        Ok(AudioTag { header, body })
    }
}

impl Encoder<AudioTag> for ByteBuffer {
    /// Encodes a AudioTag into bytes.
    fn encode(&mut self, audio_tag: &AudioTag) {
        self.encode(&audio_tag.header);
        self.put_bytes(&audio_tag.body);
    }
}

impl TryFrom<Audio> for AudioTag {
    type Error = IOError;

    fn try_from(audio: Audio) -> IOResult<Self> {
        let mut buffer: ByteBuffer = Vec::<u8>::from(audio).into();
        Decoder::<Self>::decode(&mut buffer)
    }
}

impl TryFrom<AudioTag> for Audio {
    type Error = IOError;

    fn try_from(audio_tag: AudioTag) -> IOResult<Self> {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&audio_tag);
        Ok(Self::new(buffer.into()))
    }
}

#[cfg(test)]
mod tests {
    use rand::fill;
    use super::*;

    #[test]
    fn decode_audio_tag() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u8(0);
        let mut data: [u8; 128] = [0; 128];
        fill(&mut data);
        buffer.put_bytes(&data);
        assert!(Decoder::<AudioTag>::decode(&mut buffer).is_ok())
    }

    #[test]
    fn encode_audio_tag() {
        let mut buffer = ByteBuffer::default();
        let mut expected_data: [u8; 128] = [0; 128];
        fill(&mut expected_data);
        let expected = AudioTag::new(
            AudioTagHeader::new(
                SoundFormat::LinearPcmNe,
                SoundRate::FivePointFive,
                false,
                false,
                None
            ),
            expected_data.to_vec()
        );
        buffer.encode(&expected);

        let byte = buffer.get_u8().unwrap();
        assert_eq!(SoundFormat::LinearPcmNe as u8, byte >> 4);
        assert_eq!(SoundRate::FivePointFive as u8, (byte & 0x0c) >> 2);
        assert_eq!(0, (byte & 0x02) >> 1);
        assert_eq!(0, byte & 0x01);

        let actual_data: Vec<u8> = buffer.into();
        assert_eq!(expected_data.as_slice(), actual_data)
    }
}
