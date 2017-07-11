use std::io::Read;

use {Tag, WireType};
use decode::Decode;
use decode::futures;

pub struct TagAndWireType;
impl<R: Read> Decode<R> for TagAndWireType {
    type Value = (Tag, WireType);
    type Future = futures::DecodeTagAndWireType<R>;
    fn decode(reader: R) -> Self::Future {
        futures::DecodeTagAndWireType::new(reader)
    }
}

pub struct Varint;
impl<R: Read> Decode<R> for Varint {
    type Value = u64;
    type Future = futures::DecodeVarint<R>;
    fn decode(reader: R) -> Self::Future {
        futures::DecodeVarint::new(reader)
    }
}
