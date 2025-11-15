/// Representation of first 1 byte in handshake.
///
/// Variants correspond to respectively following numbers: 
///
/// |Pattern|Number|
/// | :- | :- |
/// |`NotEncrypted` (Default)|`3`|
/// |`DiffieHellman`|`6`|
/// |`Xtea`|`8`|
/// |`Blowfish`|`9`|
/// |`Other`|other numbers|
///
/// Because of the design policy, the variant to be used actually will only be `NotEncrypted`.
/// Other variants are prepared to keep their meaning of known numbers.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    #[default]
    NotEncrypted = 3,
    DiffieHellman = 6,
    Xtea = 8,
    Blowfish = 9,
    Other = 0xff
}

impl From<u8> for EncryptionAlgorithm {
    fn from(encryption_algorithm: u8) -> Self {
        use EncryptionAlgorithm::*;

        match encryption_algorithm {
            3 => NotEncrypted,
            6 => DiffieHellman,
            8 => Xtea,
            9 => Blowfish,
            _ => Other
        }
    }
}

impl From<EncryptionAlgorithm> for u8 {
    fn from(encryption_algorithm: EncryptionAlgorithm) -> Self {
        encryption_algorithm as u8
    }
}
