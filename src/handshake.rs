//! # The structs/functions to do handshake for the RTMP layer.
//!
//! The RTMP also needs to do handshake like the TCP.
//!
//! ## The chunk format
//!
//! The chunk what will send from server-side/client-side is following respectively.
//!
//! 1. S0/C0 (1 byte)
//! 2. S1/C1 (1536 bytes)
//! 3. S2/C2 (1536 bytes)
//!
//! ### S0 and C0
//!
//! Input the RTMP version.
//!
//! * 3 (Default): The raw RTMP. This isn't encrypted neither the packet nor the network.
//! * 6, 8 and 9: The RTMPE (RTMP Encrypted). This encrypts just the packet. Its algorithm is following:
//!   * 6: Uses just the Diffie-Hellman key exchange.
//!   * 8: Uses the Diffie-Hellman key exchange with XTEA.
//!   * 9: Uses the Diffie-Hellman key exchange with Blowfish.
//!
//! However note that 6, 8 and 9 (that is, RTMPE) shouldn't use currently because these can be targetted from man-in-the-middle attack.
//! RTMPE doesn't encrypt the network.
//!
//! ### S1 and C1
//!
//! This has four segments. Input following values in them respectively:
//!
//! 1. Timestamp (4 bytes)
//! 2. Version (4 bytes)
//! 3. Random Bytes (1536 bytes)
//!
//! #### Timestamp
//!
//! The timestamp what started the RTMP handshake.
//! This hasn't been specified neither in second, in millisecond, nor in nanosecond.
//! However this may be ok to input in-second timestamp due to just four bytes.
//! This may set 0.
//!
//! #### Version
//!
//! In the official specification paper, this had been specified to input just four bytes zeroes.
//! However this has currently been input the Adobe Media Server/the Flash Player version in many of the products already to being published.
//! This is used to switch either to use HMAC-SHA256 or to use raw random bytes as them be, in the handshake.
//!
//! #### The random bytes
//!
//! The random bytes to use for the RTMP handshake.
//! In the Adobe Media Server version and above 3/the Flash Player version and above 9, this has been each other imprinted HMAC-SHA256 digest.
//! We will calculate the place where imprints its digest following formulas.
//!
//! * In the raw RTMP: `(R[0] + R[1] + R[2] + R[3]) % 778 + 12`
//! * In the RTMPE: `(R[764] + R[765] + R[766] + R[767]) % 728 + 776`
//!
//! Note where R is 1528 bytes randoms.
//! That is, above formulas aren't considering the segments which the RTMP version and the Adobe Media Server/the Flash Player version.
//!
//! The keys to use to get its digest are following.
//!
//! * Server: "Genuine Adobe Media Server 001"
//! * Client: "Genuine Adobe Flash Player 001"
//!
//! ### S2 and C2 (1536 bytes)
//!
//! This is the chunk to echo each other the S1/C1 chunk.
//! In the server, we will respond the C1 chunk.
//! In the client, we will respond the S1 chunk.
//! However in the Adobe Media Server version and above 3/the Flash Player version and above 9, we must imprint the HMAC-SHA256 signature at last 32 bytes.
//! How to create its signature is following.
//!
//! 1. Find the digest imprinted in S1/C1. We can find it by the same way as what we calclated the place to imprint the digest in S1/C1.
//! 2. Encrypt its digest by HMAC-SHA256.
//! 3. Encrypt S1/C1 by HMAC-SHA256 except last 32 bytes.
//! 4. Imprint its signature at last 32 bytes.
//!
//! Note that the server will imprint the C1 by the server keys, and the client will imprint the S1 by the client keys.
//!
//! The keys to use to get its signature is following.
//!
//! * Server: "Genuine Adobe Media Server 001" +  
//! **0x**F0EEC24A8068BEE82E00D0D1029E7E576EEC5D2D29806FAB93B8E636CFEB31AE
//! * Client: "Genuine Adobe Flash Player 001" +  
//! **0x**F0EEC24A8068BEE82E00D0D1029E7E576EEC5D2D29806FAB93B8E636CFEB31AE
//!
//! We will compare the signature which we got one and already imprinted one.
//! If they haven't coincided, it will mean we've received some invalid handshake chunk.
//! Note:
//!
//! * In the FFmpeg, above validation will fail because somehow it won't imprint its signature for the C2 chunk.
//! * In the OBS (Open Broadcaster Software), above validation hasn't been needed in the first place because it has imprinted neither its digest nor its signature.
//!
//! ## The Sending/Receiving sequence
//!
//! We will send the S0/C0 chunk, the S1/C1 chunk, and the S2/C2 chunk following order.
//!
//! 1. The client will send the C0 chunk and the C1 chunk to the server.
//! 2. When the server received the C0 chunk and the C1 chunk, the server will send the S0 chunk, the S1 chunk, and the S2 chunk to the client.
//! 3. When the client received the S0 chunk, the S1 chunk and the S2 chunk, the client will validate its digest and its signature imprinted in the S2 chunk.  
//! If they'are valid, the client will send the C2 chunk to the server.
//! 4. When the server received the C2 chunk, the server will validate its digest and its signature imprinted in the C2 chunk.  
//! If they're valid, the server will go to the phase of the application connection.
use std::{
    io::{
        Error as IOError,
        ErrorKind,
        Result as IOResult,
    },
    time::{
        Duration,
        SystemTime
    }
};
use rand::{
    prelude::*
};
use crypto::{
    hmac::{
        Hmac
    },
    mac::{
        Mac,
        MacResult
    },
    sha2::{
        Sha256
    }
};
use crate::errors::{
    ChunkLengthError,
    DigestVerificationError,
    SignatureDoesNotMatchError,
    DigestOffsetError
};

/// The size of the S0/C0 chunk.
pub(crate) const VERSION_CHUNK_SIZE: usize = 1;
/// The size of the S1/C1 and the S2/C2 chunk.
pub(crate) const HANDSHAKE_CHUNK_SIZE: usize = 1536;
const RTMP_VERSION: u8 = 3;
const DIGEST_LEN: usize = 32;
const GENUINE_FMS_KEY: &[u8] = &[
    0x47, 0x65, 0x6e, 0x75, 0x69, 0x6e, 0x65, 0x20, 0x41, 0x64,
    0x6f, 0x62, 0x65, 0x20, 0x46, 0x6c, 0x61, 0x73, 0x68, 0x20,
    0x4d, 0x65, 0x64, 0x69, 0x61, 0x20, 0x53, 0x65, 0x72, 0x76,
    0x65, 0x72, 0x20, 0x30, 0x30, 0x31, 0xf0, 0xee, 0xc2, 0x4a,
    0x80, 0x68, 0xbe, 0xe8, 0x2e, 0x00, 0xd0, 0xd1, 0x02, 0x9e,
    0x7e, 0x57, 0x6e, 0xec, 0x5d, 0x2d, 0x29, 0x80, 0x6f, 0xab,
    0x93, 0xb8, 0xe6, 0x36, 0xcf, 0xeb, 0x31, 0xae
];
const GENUINE_FP_KEY: &[u8] = &[
    0x47, 0x65, 0x6e, 0x75, 0x69, 0x6e, 0x65, 0x20, 0x41, 0x64,
    0x6f, 0x62, 0x65, 0x20, 0x46, 0x6c, 0x61, 0x73, 0x68, 0x20,
    0x50, 0x6c, 0x61, 0x79, 0x65, 0x72, 0x20, 0x30, 0x30, 0x31,
    0xf0, 0xee, 0xc2, 0x4a, 0x80, 0x68, 0xbe, 0xe8, 0x2e, 0x00,
    0xd0, 0xd1, 0x02, 0x9e, 0x7e, 0x57, 0x6e, 0xec, 0x5d, 0x2d,
    0x29, 0x80, 0x6f, 0xab, 0x93, 0xb8, 0xe6, 0x36, 0xcf, 0xeb,
    0x31, 0xae
];

#[derive(Debug)]
enum Algorithm {
    DigestOffset1,
    DigestOffset2
}

/// # The handshake validator for the RTMP
///
/// Validating the chunks what will receive from the client.
/// This will be used in the struct `RtmpHandler` currently.
#[derive(Debug)]
pub struct RtmpHandshake {
    fp9_handshake: bool,
    digest_pos_server: usize,
    start_time: Duration,
    s1: Vec<u8>,
    s2: Vec<u8>
}

impl RtmpHandshake {
    /// Constructs a new `RtmpHandshake`.
    ///
    /// # Parameters
    ///
    /// * `start_time: ::std::time::Duration`
    ///
    /// The timestamp when started to run this server.
    pub fn new(start_time: Duration) -> Self {
        RtmpHandshake {
            fp9_handshake: true,
            digest_pos_server: usize::default(),
            start_time,
            s1: Vec::new(),
            s2: Vec::new()
        }
    }

    /// Returns the current timestamp from when this server started.
    /// If the start time has been future (although it will be improbable), this will return `None`.
    pub fn get_up_time(&self) -> Option<Duration> {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().checked_sub(self.start_time)
    }

    fn generate_unversioned_handshake(&mut self, mut c0c1: Vec<u8>) -> IOResult<()> {
        let mut chunk: Vec<u8> = Vec::new();
        let up_time_bytes = (self.get_up_time().unwrap().as_secs() as u32).to_be_bytes();
        let zeroes: [u8; 1532] = [0; 1532]; // HANDSHAKE_CHUNK_SIZE + 1(range of version) - 5(range of both which are version and timestamp)

        chunk.extend_from_slice(&up_time_bytes);
        chunk.extend_from_slice(&zeroes);
        chunk.append(&mut c0c1);
        Ok(self.s1 = chunk)
    }

    /// Decodes the C0 chunk and the C1 chunk, then imprints the signature in the client chunk.
    ///
    /// # Parameters
    ///
    /// * `c0c1: Vec<u8>`
    ///
    /// The C0 chunk and the C1 chunk.
    /// Currently, these will be sent together from the client.
    ///
    /// # Errors
    ///
    /// When you got the `ChunkLengthError`:
    ///
    /// * The C0 chunk didn't find.
    ///
    /// When you got the `DigestVerificationError`:
    ///
    /// * The HMAC-SHA256 digest didn't find in the C1 chunk.
    pub fn decode_client_request1(&mut self, mut c0c1: Vec<u8>) -> IOResult<()> {
        if c0c1.len() < VERSION_CHUNK_SIZE + HANDSHAKE_CHUNK_SIZE {
            return Err(IOError::new(ErrorKind::InvalidInput, ChunkLengthError::new("Requested RTMP chunk's size is not 1537 bytes!".to_string(), None)));
        }

        c0c1 = c0c1[1..].to_vec();
        self.fp9_handshake = c0c1[4] != 0;

        if !self.fp9_handshake {
            return self.generate_unversioned_handshake(c0c1);
        }

        let mut handshake_bytes = create_handshake_bytes(self.get_up_time().unwrap());

        self.digest_pos_server = get_digest_offset(Algorithm::DigestOffset1, handshake_bytes.as_slice()).unwrap();

        let digest = calculate_digest(self.digest_pos_server, handshake_bytes.as_slice(), &GENUINE_FMS_KEY[..36]);

        self.s1.append(&mut handshake_bytes);

        for i in 0..digest.code().len() {
            self.s1[self.digest_pos_server + i] = digest.code()[i];
        }

        let mut digest_pos_client = get_digest_offset(Algorithm::DigestOffset2, c0c1.as_slice()).unwrap();

        if !verify_digest(digest_pos_client, c0c1.as_slice(), &GENUINE_FP_KEY[..30]) {
            digest_pos_client = get_digest_offset(Algorithm::DigestOffset1, c0c1.as_slice()).unwrap();

            if !verify_digest(digest_pos_client, c0c1.as_slice(), &GENUINE_FP_KEY[..30]) {
                return Err(IOError::new(ErrorKind::InvalidData, DigestVerificationError::new("Client digest verification failed.".to_string(), None)));
            }
        }

        let digest_start = digest_pos_client;
        let digest_end = digest_start + DIGEST_LEN;
        let swf_size = usize::default();

        if swf_size > 0 {
            let swf_hash: [u8; DIGEST_LEN] = [0; DIGEST_LEN];
            let swf_verification_bytes = calculate_swf_verification(self.s1.as_slice(), &swf_hash, swf_size);
        }

        let digest_resp = calculate_hmac_sha256(&c0c1[digest_start..digest_end], GENUINE_FMS_KEY);
        let signature_end = HANDSHAKE_CHUNK_SIZE - DIGEST_LEN;
        let signature_response = calculate_hmac_sha256(&c0c1[..signature_end], digest_resp.code());

        c0c1.truncate(c0c1.len() - DIGEST_LEN);
        c0c1.extend_from_slice(signature_response.code());
        Ok(self.s2 = c0c1)
    }

    /// Returns the S0 chunk, the S1 chunk and the S2 chunk for sending the response to the client.
    pub fn get_s0s1s2(&mut self) -> Vec<u8> {
        let mut s0s1s2: Vec<u8> = Vec::new();

        s0s1s2.push(RTMP_VERSION);
        s0s1s2.extend_from_slice(self.s1.as_slice());
        s0s1s2.extend_from_slice(self.s2.as_slice());
        s0s1s2
    }

    /// Decodes the C2 chunk.
    ///
    /// # Parameters
    ///
    /// * `c2: Vec<u8>`
    ///
    /// The C2 chunk.
    ///
    /// # Errors
    ///
    /// When you got the `ChunkLengthError`:
    ///
    /// * The C2 chunk length wasn't 1536 bytes.
    ///
    /// When you got the `SignatureDoesNotMatchError`:
    ///
    /// * The HMAC-SHA256 signature in the C2 chunk didn't match with stored one in the server.
    pub fn decode_client_request2(&mut self, c2: Vec<u8>) -> IOResult<()> {
        if c2.len() < HANDSHAKE_CHUNK_SIZE {
            return Err(IOError::new(ErrorKind::InvalidInput, ChunkLengthError::new("Requested RTMP chunk's size is not 1536 bytes!".to_string(), None)));
        }

        if self.fp9_handshake {
            let digest_start = self.digest_pos_server;
            let digest_end = digest_start + DIGEST_LEN;
            let digest = calculate_hmac_sha256(&self.s1[digest_start..digest_end], GENUINE_FP_KEY);
            let signature_end = HANDSHAKE_CHUNK_SIZE - DIGEST_LEN;
            let signature = calculate_hmac_sha256(&c2[..signature_end], digest.code());
            let sent_signature_start = HANDSHAKE_CHUNK_SIZE - DIGEST_LEN;
            let sent_signature_end = sent_signature_start + DIGEST_LEN;
            let sent_signature = &c2[sent_signature_start..sent_signature_end];

            if signature.code() == sent_signature {
                println!("Compatible client, handshake complete");
            } else {
                if &self.s1[8..] != &c2[8..] {
                    return Err(IOError::new(ErrorKind::InvalidData, SignatureDoesNotMatchError::new("Client signature doesn't match!".to_string(), None)));
                }
            }
        } else {
            if self.s1 != c2 {
                return Err(IOError::new(ErrorKind::InvalidData, SignatureDoesNotMatchError::new("Client signature doesn't match!".to_string(), None)));
            }
        }

        Ok(())
    }
}

fn create_handshake_bytes(up_time: Duration) -> Vec<u8> {
    let up_time_bytes = (up_time.as_secs() as u32).to_be_bytes();
    let version: [u8; 4] = [5, 0, 10, 0];
    let mut handshake_bytes: Vec<u8> = Vec::new();

    handshake_bytes.extend_from_slice(&up_time_bytes);
    handshake_bytes.extend_from_slice(&version);

    for _ in 8..HANDSHAKE_CHUNK_SIZE {
        handshake_bytes.push(random());
    }

    handshake_bytes
}

fn get_digest_offset1(handshake_bytes: &[u8]) -> IOResult<usize> {
    let mut offset = usize::default();
    let buffer_offset = 8;

    for i in 0..4 {
        offset += handshake_bytes[buffer_offset + i] as usize;
    }

    let ret = offset % 728 + 12;

    if ret + DIGEST_LEN > 771 {
        return Err(IOError::new(ErrorKind::InvalidInput, DigestOffsetError::new(format!("Invalid digest offset calc: {}", ret), None)));
    }

    Ok(ret)
}

fn get_digest_offset2(handshake_bytes: &[u8]) -> IOResult<usize> {
    let mut offset = usize::default();
    let buffer_offset = 772;

    for i in 0..4 {
        offset += handshake_bytes[buffer_offset + i] as usize;
    }

    let ret = offset % 728 + 776;

    if ret + DIGEST_LEN > 1535 {
        return Err(IOError::new(ErrorKind::InvalidInput, DigestOffsetError::new(format!("Invalid digest offset calc: {}", ret), None)));
    }

    Ok(ret)
}

fn get_digest_offset(algorithm: Algorithm, handshake_bytes: &[u8]) -> IOResult<usize> {
    match algorithm {
        Algorithm::DigestOffset2 => get_digest_offset2(handshake_bytes),
        _ => get_digest_offset1(handshake_bytes)
    }
}

fn calculate_hmac_sha256(message: &[u8], key: &[u8]) -> MacResult {
    let mut hmac = Hmac::new(Sha256::new(), key);

    hmac.input(message);
    hmac.result()
}

fn calculate_digest(digest_pos: usize, handshake_message: &[u8], key: &[u8]) -> MacResult {
    let mut message: Vec<u8> = Vec::new();

    message.extend_from_slice(&handshake_message[..digest_pos]);

    let start = digest_pos + DIGEST_LEN;
    let end = start + handshake_message.len() - digest_pos - DIGEST_LEN;

    message.extend_from_slice(&handshake_message[start..end]);
    calculate_hmac_sha256(message.as_slice(), key)
}

fn verify_digest(digest_pos: usize, handshake_message: &[u8], key: &[u8]) -> bool {
    calculate_digest(digest_pos, handshake_message, key).code() == &handshake_message[digest_pos..(digest_pos + DIGEST_LEN)]
}

fn calculate_swf_verification(handshake_message: &[u8], swf_hash: &[u8], swf_size: usize) -> Vec<u8> {
    let mut swf_hash_key: Vec<u8> = Vec::new();
    let swf_hash_key_start = HANDSHAKE_CHUNK_SIZE - DIGEST_LEN;
    let swf_hash_key_end = HANDSHAKE_CHUNK_SIZE;

    swf_hash_key.extend_from_slice(&handshake_message[swf_hash_key_start..swf_hash_key_end]);

    let bytes_form_server_hash = calculate_hmac_sha256(swf_hash, &swf_hash_key[..DIGEST_LEN]);
    let swf_size_bytes = u32::to_be_bytes(swf_size as u32);
    let mut swf_verification_bytes: Vec<u8> = Vec::new();

    swf_verification_bytes.push(1);
    swf_verification_bytes.push(1);
    swf_verification_bytes.extend_from_slice(&swf_size_bytes);
    swf_verification_bytes.extend_from_slice(&swf_size_bytes);
    swf_verification_bytes.extend_from_slice(bytes_form_server_hash.code());
    swf_verification_bytes
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_digest_offset() {
        let mut v: Vec<u8> = Vec::new();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let flash_version: [u8; 4] = [9, 0, 124, 2];
        let digest_offsets: [u8; 4] = [1, 1, 1, 1];

        v.extend_from_slice(&(timestamp.as_secs() as u32).to_be_bytes());
        v.extend_from_slice(&flash_version);
        v.extend_from_slice(&digest_offsets);

        for _ in 8..(HANDSHAKE_CHUNK_SIZE - digest_offsets.len()) {
            v.push(random());
        }

        let digest_pos1 = get_digest_offset(Algorithm::DigestOffset1, v.as_slice());

        v = Vec::new();
        v.extend_from_slice(&(timestamp.as_secs() as u32).to_be_bytes());
        v.extend_from_slice(&flash_version);

        for _ in 8..HANDSHAKE_CHUNK_SIZE {
            v.push(random());
        }

        for i in 0..digest_offsets.len() {
            v[772 + i] = digest_offsets[i];
        }

        let digest_pos2 = get_digest_offset(Algorithm::DigestOffset2, v.as_slice());

        assert_eq!((1 + 1 + 1 + 1) % 728 + 12, digest_pos1.unwrap());
        assert_eq!((1 + 1 + 1 + 1) % 728 + 776, digest_pos2.unwrap())
    }

    #[test]
    fn test_decode_client_request() {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let mut handshake = RtmpHandshake::new(timestamp);
        let mut c0c1: Vec<u8> = Vec::new();
        let flash_version: [u8; 4] = [9, 0, 124, 2];

        c0c1.push(RTMP_VERSION);
        c0c1.extend_from_slice(&(timestamp.as_secs() as u32).to_be_bytes());
        c0c1.extend_from_slice(&flash_version);

        for _ in 8..HANDSHAKE_CHUNK_SIZE {
            c0c1.push(random());
        }

        let digest_pos_client = get_digest_offset(Algorithm::DigestOffset1, &c0c1[1..]).unwrap();
        let digest_client = calculate_digest(digest_pos_client, &c0c1[1..], &GENUINE_FP_KEY[..30]);

        for i in 0..digest_client.code().len() {
            c0c1[digest_pos_client + i + 1] = digest_client.code()[i];
        }

        let result1 = handshake.decode_client_request1(c0c1);

        assert!(result1.is_ok());

        let s0s1s2 = handshake.get_s0s1s2();
        let mut c2 = s0s1s2[1..(HANDSHAKE_CHUNK_SIZE + 1)].to_vec();
        let digest_pos_server = get_digest_offset(Algorithm::DigestOffset1, c2.as_slice()).unwrap();
        let digest_server = calculate_hmac_sha256(&c2[digest_pos_server..(digest_pos_server + DIGEST_LEN)], GENUINE_FP_KEY);
        let signature_server = calculate_hmac_sha256(&c2[..(HANDSHAKE_CHUNK_SIZE - DIGEST_LEN)], digest_server.code());

        for i in 0..signature_server.code().len() {
            c2[HANDSHAKE_CHUNK_SIZE - DIGEST_LEN + i] = signature_server.code()[i];
        }

        let result2 = handshake.decode_client_request2(c2);

        assert!(result2.is_ok())
    }
}
