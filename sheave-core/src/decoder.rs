use std::io::Result as IOResult;

pub trait Decoder<T> {
    fn decode(&mut self) -> IOResult<T>;
}
