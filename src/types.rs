use std::marker::PhantomData;

use traits::{Type, Field, Pattern};
use wire::WireType;

pub struct Bool;
impl Pattern for Bool {
    type Value = bool;
}
impl Type for Bool {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Int32;
impl Pattern for Int32 {
    type Value = i32;
}
impl Type for Int32 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Int64;
impl Pattern for Int64 {
    type Value = i64;
}
impl Type for Int64 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Uint32;
impl Pattern for Uint32 {
    type Value = u32;
}
impl Type for Uint32 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Uint64;
impl Pattern for Uint64 {
    type Value = u64;
}
impl Type for Uint64 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Sint32;
impl Pattern for Sint32 {
    type Value = i32;
}
impl Type for Sint32 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Sint64;
impl Pattern for Sint64 {
    type Value = i64;
}
impl Type for Sint64 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Fixed32;
impl Pattern for Fixed32 {
    type Value = u32;
}
impl Type for Fixed32 {
    fn wire_type() -> WireType {
        WireType::Bit32
    }
}

pub struct Fixed64;
impl Pattern for Fixed64 {
    type Value = u64;
}
impl Type for Fixed64 {
    fn wire_type() -> WireType {
        WireType::Bit64
    }
}

pub struct Sfixed32;
impl Pattern for Sfixed32 {
    type Value = i32;
}
impl Type for Sfixed32 {
    fn wire_type() -> WireType {
        WireType::Bit32
    }
}

pub struct Sfixed64;
impl Pattern for Sfixed64 {
    type Value = i64;
}
impl Type for Sfixed64 {
    fn wire_type() -> WireType {
        WireType::Bit64
    }
}

pub struct Bytes;
impl Pattern for Bytes {
    type Value = Vec<u8>;
}
impl Type for Bytes {
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}

pub struct Str;
impl Pattern for Str {
    type Value = String;
}
impl Type for Str {
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}

pub struct Message<Fields>(PhantomData<Fields>);
impl<A, B> Pattern for Message<(A, B)>
where
    A: Pattern,
    B: Pattern,
{
    type Value = (A::Value, B::Value);
}
impl<A, B> Type for Message<(A, B)>
where
    A: Field,
    B: Field,
{
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}
