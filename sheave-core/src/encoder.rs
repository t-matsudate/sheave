pub trait Encoder<T> {
    fn encode(&mut self, value: &T);
}
