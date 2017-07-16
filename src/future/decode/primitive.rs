use std::io::Read;

use Decode;
use future::read::{ReadBytes, ReadAllBytes};

impl<R: Read> Decode<R> for [u8; 4] {
    type Future = ReadBytes<R, Self>;
    fn decode(reader: R) -> Self::Future {
        ReadBytes::new(reader, [0; 4])
    }
}

impl<R: Read> Decode<R> for [u8; 8] {
    type Future = ReadBytes<R, Self>;
    fn decode(reader: R) -> Self::Future {
        ReadBytes::new(reader, [0; 8])
    }
}

impl<R: Read> Decode<R> for Vec<u8> {
    type Future = ReadAllBytes<R>;
    fn decode(reader: R) -> Self::Future {
        ReadAllBytes::new(reader)
    }
}
