//! Scalar Value Types.
//!
//! https://developers.google.com/protocol-buffers/docs/proto3#scalar
use {Type, WireType};

#[derive(Debug, Clone, Copy)]
pub struct Double;
impl Type for Double {
    type Value = f64;
    fn wire_type() -> WireType {
        WireType::Bit64
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Float;
impl Type for Float {
    type Value = f32;
    fn wire_type() -> WireType {
        WireType::Bit32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Int32;
impl Type for Int32 {
    type Value = i32;
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Int64;
impl Type for Int64 {
    type Value = i64;
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Uint32;
impl Type for Uint32 {
    type Value = u32;
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Uint64;
impl Type for Uint64 {
    type Value = u64;
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sint32;
impl Type for Sint32 {
    type Value = i32;
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sint64;
impl Type for Sint64 {
    type Value = i64;
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fixed32;
impl Type for Fixed32 {
    type Value = u32;
    fn wire_type() -> WireType {
        WireType::Bit32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fixed64;
impl Type for Fixed64 {
    type Value = u64;
    fn wire_type() -> WireType {
        WireType::Bit64
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sfixed32;
impl Type for Sfixed32 {
    type Value = i32;
    fn wire_type() -> WireType {
        WireType::Bit32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sfixed64;
impl Type for Sfixed64 {
    type Value = i64;
    fn wire_type() -> WireType {
        WireType::Bit64
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bool;
impl Type for Bool {
    type Value = bool;
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Str;
impl Type for Str {
    type Value = String;
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bytes;
impl Type for Bytes {
    type Value = Vec<u8>;
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}
