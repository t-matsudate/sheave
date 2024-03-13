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
    Aac = 10,
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
            10 => Aac,
            11 => Speex,
            14 => Mp3_8,
            15 => DeviceSpecific,
            _ => panic!("Unreachable sound format.")
        }
    }
}

impl From<SoundFormat> for u8 {
    fn from(sound_format: SoundFormat) -> Self {
        sound_format as u8
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundRate {
    FivePointFive,
    Eleven,
    TwnetyTwo,
    FourtyFour
}

impl From<u8> for SoundRate {
    fn from(sound_rate: u8) -> Self {
        use SoundRate::*;

        match sound_rate {
            0 => FivePointFive,
            1 => Eleven,
            2 => TwnetyTwo,
            3 => FourtyFour,
            _ => panic!("Unreachable sound rate.")
        }
    }
}

impl From<SoundRate> for u8 {
    fn from(sound_rate: SoundRate) -> Self {
        sound_rate as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioTagHeader {
    sound_format: SoundFormat,
    sound_rate: SoundRate,
    is_sixteen_bits: bool,
    is_stereo: bool,
    is_aac_raw: Option<bool>
}

impl AudioTagHeader {
    pub fn new(sound_format: SoundFormat, sound_rate: SoundRate, is_sixteen_bits: bool, is_stereo: bool, is_aac_raw: Option<bool>) -> Self {
        Self { sound_format, sound_rate, is_sixteen_bits, is_stereo, is_aac_raw }
    }
}

impl Decoder<AudioTagHeader> for ByteBuffer {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioTag {
    header: AudioTagHeader,
    body: Vec<u8>
}

impl AudioTag {
    pub fn new(header: AudioTagHeader, body: Vec<u8>) -> Self {
        Self { header, body }
    }
}

impl Decoder<AudioTag> for ByteBuffer {
    fn decode(&mut self) -> IOResult<AudioTag> {
        let header: AudioTagHeader = self.decode()?;
        let remained = self.remained();
        let body = self.get_bytes(remained)?.to_vec();

        Ok(AudioTag { header, body })
    }
}

impl Encoder<AudioTag> for ByteBuffer {
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
    use rand::{
        Fill,
        thread_rng
    };
    use super::*;

    #[test]
    fn decode_audio_tag() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u8(0);
        let mut data: [u8; 128] = [0; 128];
        data.try_fill(&mut thread_rng()).unwrap();
        buffer.put_bytes(&data);
        assert!(Decoder::<AudioTag>::decode(&mut buffer).is_ok())
    }

    #[test]
    fn encode_audio_tag() {
        let mut buffer = ByteBuffer::default();
        let mut expected_data: [u8; 128] = [0; 128];
        expected_data.try_fill(&mut thread_rng()).unwrap();
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
