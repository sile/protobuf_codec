use std::marker::PhantomData;

use traits::{Type, Field};
use wire::WireType;

pub struct Bool;
impl Type for Bool {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Int32;
impl Type for Int32 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Int64;
impl Type for Int64 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Uint32;
impl Type for Uint32 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Uint64;
impl Type for Uint64 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Sint32;
impl Type for Sint32 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Sint64;
impl Type for Sint64 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Fixed32;
impl Type for Fixed32 {
    fn wire_type() -> WireType {
        WireType::Bit32
    }
}

pub struct Fixed64;
impl Type for Fixed64 {
    fn wire_type() -> WireType {
        WireType::Bit64
    }
}

pub struct Sfixed32;
impl Type for Sfixed32 {
    fn wire_type() -> WireType {
        WireType::Bit32
    }
}

pub struct Sfixed64;
impl Type for Sfixed64 {
    fn wire_type() -> WireType {
        WireType::Bit64
    }
}

pub struct Bytes;
impl Type for Bytes {
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}

pub struct Str;
impl Type for Str {
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}

pub struct Message<Fields>(PhantomData<Fields>);
impl<A, B> Type for Message<(A, B)>
where
    A: Field,
    B: Field,
{
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}
