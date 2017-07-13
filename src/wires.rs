//! Wire Value Types.
//!
//! https://developers.google.com/protocol-buffers/docs/encoding#structure
use {Type, Payload, WireType};

#[derive(Debug, Clone, Copy)]
pub struct Varint;
impl Type for Varint {
    type Value = u64;
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bit32;
impl Type for Bit32 {
    type Value = [u8; 4];
    fn wire_type() -> WireType {
        WireType::Bit32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bit64;
impl Type for Bit64 {
    type Value = [u8; 8];
    fn wire_type() -> WireType {
        WireType::Bit64
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LengthDelimited<T>(pub T);
impl<T: Payload> Type for LengthDelimited<T> {
    type Value = T::Value;
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}
