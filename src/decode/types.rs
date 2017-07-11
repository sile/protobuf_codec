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

#[derive(Debug, Clone, Copy)]
pub struct Bool;
impl<R: Read> Decode<R> for Bool {
    type Value = bool;
    type Future = futures::DecodeBool<R>;
    fn decode(reader: R) -> Self::Future {
        futures::DecodeBool::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fixed32;
impl<R: Read> Decode<R> for Fixed32 {
    type Value = u32;
    type Future = futures::DecodeFixed32<R>;
    fn decode(reader: R) -> Self::Future {
        futures::DecodeFixed32::new(reader)
    }
}

// TODO: delete
pub struct Varint;
impl<R: Read> Decode<R> for Varint {
    type Value = u64;
    type Future = futures::DecodeVarint<R>;
    fn decode(reader: R) -> Self::Future {
        futures::DecodeVarint::new(reader)
    }
}
impl<R: Read> Decode<R> for [u8; 4] {
    type Value = Self;
    type Future = futures::ReadBytes<R, Self>;
    fn decode(reader: R) -> Self::Future {
        futures::ReadBytes::new(reader, [0; 4])
    }
}
impl<R: Read> Decode<R> for [u8; 8] {
    type Value = Self;
    type Future = futures::ReadBytes<R, Self>;
    fn decode(reader: R) -> Self::Future {
        futures::ReadBytes::new(reader, [0; 8])
    }
}

#[cfg(test)]
mod test {
    use futures::Future;

    use decode::Decode;
    use super::*;

    #[test]
    fn decode_bool() {
        let input = [0b0000_0001];
        let (_, b) = track_try_unwrap!(Bool::decode(&input[..]).wait());
        assert_eq!(b, true);

        let input = [0b0000_0000];
        let (_, b) = track_try_unwrap!(Bool::decode(&input[..]).wait());
        assert_eq!(b, false);
    }

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
