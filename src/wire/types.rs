//! Wire Value Types.
//!
//! [protocol-buffers/encoding#structure](https://developers.google.com/protocol-buffers/docs/encoding#structure)
#[derive(Debug)]
pub struct Varint(pub u64);
impl From<u64> for Varint {
    fn from(f: u64) -> Self {
        Varint(f)
    }
}

#[derive(Debug)]
pub struct Bit32(pub [u8; 4]);
impl From<[u8; 4]> for Bit32 {
    fn from(f: [u8; 4]) -> Self {
        Bit32(f)
    }
}

#[derive(Debug)]
pub struct Bit64(pub [u8; 8]);
impl From<[u8; 8]> for Bit64 {
    fn from(f: [u8; 8]) -> Self {
        Bit64(f)
    }
}

#[derive(Debug)]
pub struct LengthDelimited<T>(pub T);
impl<T> From<T> for LengthDelimited<T> {
    fn from(f: T) -> Self {
        LengthDelimited(f)
    }
}
