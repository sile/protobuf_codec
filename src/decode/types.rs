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

#[cfg(test)]
mod test {
    use futures::Future;

    use decode::Decode;
    use super::*;

    #[test]
    fn decode_varint() {
        let input = [0b0000_0001];
        let (_, n) = track_try_unwrap!(Varint::decode(&input[..]).wait());
        assert_eq!(n, 1);

        let input = [0b1010_1100, 0b0000_0010];
        let (_, n) = track_try_unwrap!(Varint::decode(&input[..]).wait());
        assert_eq!(n, 300);
    }
}
