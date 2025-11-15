use std::io::Result as IOResult;

/// Decodes bytes into a chunk data.
///
/// This can return errors for decoding fails by something cause.
pub trait Decoder<T> {
    fn decode(&mut self) -> IOResult<T>;
}
