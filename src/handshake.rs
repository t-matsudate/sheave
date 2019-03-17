use std::{
    io::{
        Error as IOError,
        ErrorKind,
        Read,
        Result as IOResult,
        Write
    },
    mem::{
        transmute
    },
    net::{
        TcpStream
    },
    time::{
        Duration,
        SystemTime
    }
};
use log::{
    LogLevel,
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
use self::errors::{
    ChunkLengthError,
    DigestVerificationError,
    SignatureDoesNotMatchError,
    DigestOffsetError 
}

const VERSION_CHUNK_SIZE: usize = 1;
const HANDSHAKE_CHUNK_SIZE: usize = 1536;
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
pub(self) enum Algorithm {
    DigestOffset1,
    DigestOffset2
}

#[repr(u8)]
#[derive(Debug)]
pub(crate) enum RtmpState {
    Connect,
    Handshake,
    Connected,
    Error,
    Disconnecting,
    Disconnected
}

#[derive(Debug)]
pub(crate) struct RtmpHandshake {
    s1: Vec<u8>,
    start_time: Duration,
    state: RtmpState,
    digest_pos_server: usize,
    fp9_handshake: bool
}

impl RtmpHandshake {
    pub(crate) fn new(start_time: Duration) -> Self {
        RtmpHandshake {
            s1: Vec::new(),
            start_time,
            state: RtmpState::Connect,
            digest_pos_server: usize::default(),
            fp9_handshake: true
        }
    }

    pub(crate) fn set_rtmp_state(&mut self, state: RtmpState) {
        self.state = state;
    }

    pub(crate) fn get_rtmp_state(&self) -> &RtmpState {
        &self.state
    }

    fn get_up_time(&self) ->  Option<Duration> {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().checked_sub(self.start_time)
    }

    fn generate_unversioned_handshake(&mut self, c1: &[u8], stream: &mut TcpStream) -> IOResult<()> {
        debug!("Using old style (un-versioned) handshake.");

        let mut chunk: Vec<u8> = Vec::new();
        let mut up_time_bytes = unsafe { transmute::<u32, [u8; 4]>(self.get_up_time().unwrap().as_secs() as u32) };
        let zeroes: [u8; 1532] = [0; 1532]; // HANDSHAKE_CHUNK_SIZE + 1(range of version) - 5(range of both which are version and timestamp)

        up_time_bytes.reverse();
        chunk.push(RTMP_VERSION);
        chunk.extend_from_slice(&up_time_bytes);
        chunk.extend_from_slice(&zeroes);
        chunk.extend_from_slice(c1);
        self.s1 = chunk[1..].to_vec();
        stream.write_all(chunk.as_slice())
    }

    pub(crate) fn decode_client_request1(&mut self, stream: &mut TcpStream) -> IOResult<()> {
        let mut c1: Vec<u8> = Vec::new();
        let mut chunk: [u8; VERSION_CHUNK_SIZE + HANDSHAKE_CHUNK_SIZE] = [0; VERSION_CHUNK_SIZE + HANDSHAKE_CHUNK_SIZE];

        if stream.read(&mut chunk)? != VERSION_CHUNK_SIZE + HANDSHAKE_CHUNK_SIZE {
            return Err(IOError::new(ErrorKind::InvalidInput, ChunkLengthError::new("Requested RTMP chunk's size is not 1537 bytes!".to_string(), None)));
        } else {
            c1.extend_from_slice(&chunk);
            debug!("decode_handshake_c0c1 - buffer: {:x?}", c1);
        }

        trace!("Incoming C0 connection type: {}", c1[0]);
        self.set_rtmp_state(RtmpState::Handshake);
        c1.remove(0);

        if log_enabled!(LogLevel::Trace) {
            trace!("decode_client_request1: {:x?}", c1);
        }

        if log_enabled!(LogLevel::Debug) {
            debug!("Flash player version: {:x?}", &c1[4..8]);
        }

        self.fp9_handshake = c1[4] != 0;

        if !self.fp9_handshake {
            return self.generate_unversioned_handshake(c1.as_slice(), stream);
        }

        let mut handshake_bytes = create_handshake_bytes(self.get_up_time().unwrap());

        if log_enabled!(LogLevel::Debug) {
            debug!("Server handshake bytes: {:x?}", handshake_bytes);
        }

        self.digest_pos_server = get_digest_offset(Algorithm::DigestOffset2, handshake_bytes.as_slice()).unwrap();
        debug!("Server digest position offset: {} algorithm: {:?}", self.digest_pos_server, Algorithm::DigestOffset2);

        let digest = calculate_digest(self.digest_pos_server, handshake_bytes.as_slice(), &GENUINE_FMS_KEY[..36]);

        self.s1.append(&mut handshake_bytes);

        for i in 0..digest.code().len() {
            self.s1[self.digest_pos_server + i] = digest.code()[i];
        }

        debug!("Server digest: {:x?}", &self.s1[self.digest_pos_server..(self.digest_pos_server + DIGEST_LEN)]);
        trace!("Trying algorithm: {:?}", Algorithm::DigestOffset2);

        let mut digest_pos_client = get_digest_offset(Algorithm::DigestOffset2, c1.as_slice()).unwrap();

        debug!("Client digest position offset: {}", digest_pos_client);

        if !verify_digest(digest_pos_client, c1.as_slice(), &GENUINE_FP_KEY[..30]) {
            trace!("Trying algorithm: {:?}", Algorithm::DigestOffset1);
            digest_pos_client = get_digest_offset(Algorithm::DigestOffset1, c1.as_slice()).unwrap();
            debug!("Client digest position offset: {}", digest_pos_client);

            if !verify_digest(digest_pos_client, c1.as_slice(), &GENUINE_FP_KEY[..30]) {
                warn!("Client digest verification failed.");
                return Err(IOError::new(ErrorKind::InvalidInput, DigestVerificationError::new("Client digest verification failed.".to_string(), None)));
            }
        }

        let digest_start = digest_pos_client;
        let digest_end = digest_start + DIGEST_LEN;

        debug!("Client digest: {:x?}", &c1[digest_start..digest_end]);

        let swf_size = usize::default();

        if swf_size > 0 {
            let swf_hash: [u8; DIGEST_LEN] = [0; DIGEST_LEN];
            let swf_verification_bytes = calculate_swf_verification(self.s1.as_slice(), &swf_hash, swf_size);

            debug!("Swf digest: {:x?}", swf_verification_bytes);
        }

        let digest_resp = calculate_hmac_sha256(&c1[digest_start..digest_end], GENUINE_FMS_KEY);
        let signature_end = HANDSHAKE_CHUNK_SIZE - DIGEST_LEN;
        let signature_response = calculate_hmac_sha256(&c1[..signature_end], digest_resp.code());

        debug!("Digest response (key): {:x?}", digest_resp.code());
        debug!("Signature response: {:x?}", signature_response.code());
        c1.truncate(c1.len() - DIGEST_LEN);
        c1.extend_from_slice(signature_response.code());

        let mut s0s1s2: Vec<u8> = Vec::new();

        s0s1s2.push(RTMP_VERSION);
        s0s1s2.extend_from_slice(self.s1.as_slice());
        s0s1s2.append(&mut c1);

        if log_enabled!(LogLevel::Trace) {
            trace!("S0+S1+S2 size: {}", s0s1s2.len());
        }

        stream.write(s0s1s2.as_slice()).map(|_| ())
    }

    pub(crate) fn decode_client_request2(&mut self, stream: &mut TcpStream) -> IOResult<()> {
        let mut c2: Vec<u8> = Vec::new();
        let mut chunk: [u8; HANDSHAKE_CHUNK_SIZE] = [0; HANDSHAKE_CHUNK_SIZE];

        if stream.read(&mut chunk)? != HANDSHAKE_CHUNK_SIZE {
            return Err(IOError::new(ErrorKind::InvalidInput, ChunkLengthError::new("Requested RTMP chunk's size is not 1536 bytes!".to_string(), None)));
        } else {
            c2.extend_from_slice(&chunk);
            debug!("decode_handshake_c2 - buffer: {:x?}", c2);
        }

        if log_enabled!(LogLevel::Debug) {
            debug!("decode_client_request2: {:x?}", c2);
        }

        if self.fp9_handshake {
            let digest_start = self.digest_pos_server;
            let digest_end = digest_start + DIGEST_LEN;
            let digest = calculate_hmac_sha256(&self.s1[digest_start..digest_end], GENUINE_FP_KEY);
            let signature_end = HANDSHAKE_CHUNK_SIZE - DIGEST_LEN;
            let signature = calculate_hmac_sha256(&c2[..signature_end], digest.code());

            if log_enabled!(LogLevel::Debug) {
                debug!("Digest key: {:x?}", digest.code());
                debug!("Signature calculated: {:x?}", signature.code());
            }

            let sent_signature_start = HANDSHAKE_CHUNK_SIZE - DIGEST_LEN;
            let sent_signature_end = sent_signature_start + DIGEST_LEN;
            let sent_signature = &c2[sent_signature_start..sent_signature_end];

            if log_enabled!(LogLevel::Debug) {
                debug!("Client sent signature: {:x?}", sent_signature);
            }

            if signature.code() == sent_signature {
                debug!("Compatible client, handshake complete");
            } else {
                warn!("Client not compatible");

                if &self.s1[8..] != &c2[8..] {
                    return Err(IOError::new(ErrorKind::InvalidData, SignatureDoesNotMatchError::new("Client signature doesn't match!".to_string(), None)));
                }
            }
        } else {
            if self.s1 != c2 {
                info!("Client signature doesn't match!");
                return Err(IOError::new(ErrorKind::InvalidData, SignatureDoesNotMatchError::new("Client signature doesn't match!".to_string(), None)));
            }
        }

        Ok(())
    }
}

fn create_handshake_bytes(up_time: Duration) -> Vec<u8> {
    let mut up_time_bytes = unsafe { transmute::<u32, [u8; 4]>(up_time.as_secs() as u32) };
    let version: [u8; 4] = [5, 0, 15, 0];
    let mut handshake_bytes: Vec<u8> = Vec::new();

    up_time_bytes.reverse();
    handshake_bytes.extend_from_slice(&up_time_bytes);
    handshake_bytes.extend_from_slice(&version);

    for _ in 8..HANDSHAKE_CHUNK_SIZE {
        handshake_bytes.push(random::<u8>());
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
        error!("Invalid digest offset calc: {}", ret);
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
        error!("Invalid digest offset calc: {}", ret);
        return Err(IOError::new(ErrorKind::InvalidInput, DigestOffsetError::new(format!("Invalid digest offset calc: {}", ret), None)));
    }

    Ok(ret)
}

pub(self) fn get_digest_offset(algorithm: Algorithm, handshake_bytes: &[u8]) -> IOResult<usize> {
    match algorithm {
        Algorithm::DigestOffset2 => get_digest_offset2(handshake_bytes),
        _ => get_digest_offset1(handshake_bytes)
    }
}

pub(self) fn calculate_hmac_sha256(message: &[u8], key: &[u8]) -> MacResult {
    if log_enabled!(LogLevel::Trace) {
        trace!("calculate_hmac_sha256 - message_len: {}", message.len());
        trace!("calculate_hmac_sha256 - message: {:x?}", message);
        trace!("calculate_hmac_sha256 - key_len: {} key: {:x?}", key.len(), key);
    }

    let mut hmac = Hmac::new(Sha256::new(), key);

    hmac.input(message);
    hmac.result()
}

pub(self) fn calculate_digest(digest_pos: usize, handshake_message: &[u8], key: &[u8]) -> MacResult {
    if log_enabled!(LogLevel::Trace) {
        trace!("calculate_digest - digest_pos: {} key_len: {}", digest_pos, key.len());
    }

    let mut message: Vec<u8> = Vec::new();

    message.extend_from_slice(&handshake_message[..digest_pos]);

    let start = digest_pos + DIGEST_LEN;
    let end = start + handshake_message.len() - digest_pos - DIGEST_LEN;

    message.extend_from_slice(&handshake_message[start..end]);
    calculate_hmac_sha256(message.as_slice(), key)
}

fn verify_digest(digest_pos: usize, handshake_message: &[u8], key: &[u8]) -> bool {
    if log_enabled!(LogLevel::Trace) {
        trace!("verify_digest - digest_pos: {} key_len: {} handshake_size: {}", digest_pos, key.len(), handshake_message.len());
    }

    calculate_digest(digest_pos, handshake_message, key).code() == &handshake_message[digest_pos..(digest_pos + DIGEST_LEN)]
}

fn calculate_swf_verification(handshake_message: &[u8], swf_hash: &[u8], swf_size: usize) -> Vec<u8> {
    let mut swf_hash_key: Vec<u8> = Vec::new();
    let swf_hash_key_start = HANDSHAKE_CHUNK_SIZE - DIGEST_LEN;
    let swf_hash_key_end = HANDSHAKE_CHUNK_SIZE;

    swf_hash_key.extend_from_slice(&handshake_message[swf_hash_key_start..swf_hash_key_end]);

    let bytes_form_server_hash = calculate_hmac_sha256(swf_hash, &swf_hash_key[..DIGEST_LEN]);
    let mut swf_size_bytes = unsafe { transmute::<u32, [u8; 4]>(swf_size as u32) };
    let mut swf_verification_bytes: Vec<u8> = Vec::new();

    swf_size_bytes.reverse();
    swf_verification_bytes.push(1);
    swf_verification_bytes.push(1);
    swf_verification_bytes.extend_from_slice(&swf_size_bytes);
    swf_verification_bytes.extend_from_slice(&swf_size_bytes);
    swf_verification_bytes.extend_from_slice(bytes_form_server_hash.code());
    debug!("initialized swf verification response from swfSize: {} swfHash:\n{:?}\n{:?}", swf_size, swf_hash, swf_verification_bytes);
    swf_verification_bytes
}

/*
#[cfg(test)]
mod test {
    use super::*;

    fn create_handshake_bytes() -> Vec<u8> {
        let up_time_bytes: [u8; 4] = [0, 0, 0, 0];
        let version: [u8; 4] = [9, 0, 124, 2];
        let mut handshake_bytes: Vec<u8> = Vec::new();

        handshake_bytes.extend_from_slice(&up_time_bytes);
        handshake_bytes.extend_from_slice(&version);

        for _ in 8..HANDSHAKE_CHUNK_SIZE {
            handshake_bytes.push(random::<u8>());
        }

        handshake_bytes
    }

    #[test]
    fn decode_client_request_test() {
        let mut stream = TcpStream::connect("127.0.0.1:1935").unwrap();
        let mut handshake_bytes = create_handshake_bytes();
        let digest_pos_client = get_digest_offset(Algorithm::DigestOffset1, &handshake_bytes[1..]).unwrap();
        let digest_client = calculate_digest(digest_pos_client, &handshake_bytes[1..], &GENUINE_FP_KEY[..30]);

        for i in 0..digest_client.code().len() {
            handshake_bytes[digest_pos_client + i + 1] = digest_client.code()[i];
        }

        let mut s0s1s2: [u8; HANDSHAKE_CHUNK_SIZE * 2 + VERSION_CHUNK_SIZE] = [0; HANDSHAKE_CHUNK_SIZE * 2 + VERSION_CHUNK_SIZE];

        assert!(stream.write(handshake_bytes.as_slice()).and_then(|_| stream.read(&mut s0s1s2)).is_ok());
        handshake_bytes.remove(0);

        let s1_start = VERSION_CHUNK_SIZE;
        let s1_end = s1_start + HANDSHAKE_CHUNK_SIZE;
        let s2_start = s1_end;
        let s2_end = s2_start + HANDSHAKE_CHUNK_SIZE;
        let s0 = s0s1s2[0];
        let s1 = &s0s1s2[s1_start..s1_end];
        let s2 = &s0s1s2[s2_start..s2_end];

        if s0 >= 3 {
            let digest_pos_server = get_digest_offset(Algorithm::DigestOffset2, s1).or(get_digest_offset(Algorithm::DigestOffset1, s1)).unwrap();
            let digest_server_start = digest_pos_server;
            let digest_server_end = digest_server_start + DIGEST_LEN;
            let mut digest_server = calculate_hmac_sha256(&handshake_bytes[digest_pos_client..(digest_pos_client + DIGEST_LEN)], GENUINE_FMS_KEY);
            let mut signature_server = calculate_hmac_sha256(&s2[..(HANDSHAKE_CHUNK_SIZE - DIGEST_LEN)], digest_server.code());

            assert_eq!(signature_server.code(), &s2[(HANDSHAKE_CHUNK_SIZE - DIGEST_LEN)..]);
            handshake_bytes = Vec::new();

            for _ in 0..HANDSHAKE_CHUNK_SIZE {
                handshake_bytes.push(random::<u8>());
            }

            digest_server = calculate_hmac_sha256(&s1[digest_server_start..digest_server_end], GENUINE_FP_KEY);
            signature_server = calculate_hmac_sha256(&handshake_bytes[..(HANDSHAKE_CHUNK_SIZE - DIGEST_LEN)], digest_server.code());

            for i in 0..signature_server.code().len() {
                handshake_bytes[HANDSHAKE_CHUNK_SIZE - DIGEST_LEN - i] = signature_server.code()[i];
            }

            assert!(stream.write(handshake_bytes.as_slice()).is_ok());
        } else {
            assert!(stream.write(s1).is_ok());
        }
    }
}
*/
