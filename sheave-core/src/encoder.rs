/// Encodes a chunk data into bytes.
pub trait Encoder<T> {
    fn encode(&mut self, value: &T);
}
