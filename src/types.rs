use std;
use byteorder::{ByteOrder, LittleEndian};
use trackable::error::ErrorKindExt;

use {Result, ErrorKind};
use traits::{FieldType, TryFrom, Message, Packable};
use wire::WireType;
use wire::types::{Varint, Bit32, Bit64, LengthDelimited};

macro_rules! impl_scalar_type {
    ($t:ident, $base:ty, $wire:ident) => {
        impl FieldType for $t {
            fn wire_type() -> WireType {
                WireType::$wire
            }
        }
        impl From<$base> for $t {
            fn from(f: $base) -> Self {
                $t(f)
            }
        }
        impl From<$t> for $base {
            fn from(f: $t) -> Self {
                f.0
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Bool(pub bool);
impl_scalar_type!(Bool, bool, Varint);
impl Packable for Bool {}
impl TryFrom<Varint> for Bool {
    fn try_from(f: Varint) -> Result<Self> {
        track_assert!(
            f.0 <= 1,
            ErrorKind::Invalid,
            "Tool large `bool` value: {}",
            f.0
        );
        Ok(Bool(f.0 != 0))
    }
}

#[derive(Debug, Default)]
pub struct Int32(pub i32);
impl_scalar_type!(Int32, i32, Varint);
impl Packable for Int32 {}
impl TryFrom<Varint> for Int32 {
    fn try_from(f: Varint) -> Result<Self> {
        let n = f.0 as i64;
        track_assert!(
            n >= std::i32::MIN as i64,
            ErrorKind::Invalid,
            "Tool small `int32` value: {}",
            n
        );
        track_assert!(
            n <= std::i32::MAX as i64,
            ErrorKind::Invalid,
            "Tool large `int32` value: {}",
            n
        );
        Ok(Int32(f.0 as i32))
    }
}

#[derive(Debug, Default)]
pub struct Int64(pub i64);
impl_scalar_type!(Int64, i64, Varint);
impl Packable for Int64 {}
impl From<Varint> for Int64 {
    fn from(f: Varint) -> Self {
        Int64(f.0 as i64)
    }
}

#[derive(Debug, Default)]
pub struct Uint32(pub u32);
impl_scalar_type!(Uint32, u32, Varint);
impl Packable for Uint32 {}
impl TryFrom<Varint> for Uint32 {
    fn try_from(f: Varint) -> Result<Self> {
        track_assert!(
            f.0 <= std::u32::MAX as u64,
            ErrorKind::Invalid,
            "Tool large `uint32` value: {}",
            f.0
        );
        Ok(Uint32(f.0 as u32))
    }
}

#[derive(Debug, Default)]
pub struct Uint64(pub u64);
impl_scalar_type!(Uint64, u64, Varint);
impl Packable for Uint64 {}
impl From<Varint> for Uint64 {
    fn from(f: Varint) -> Self {
        Uint64(f.0)
    }
}

#[derive(Debug, Default)]
pub struct Sint32(pub i32);
impl_scalar_type!(Sint32, i32, Varint);
impl Packable for Sint32 {}
impl TryFrom<Varint> for Sint32 {
    fn try_from(f: Varint) -> Result<Self> {
        let n = ((f.0 << 63) | (f.0 >> 1)) as i64;
        track_assert!(
            n >= std::i32::MIN as i64,
            ErrorKind::Invalid,
            "Tool small `int32` value: {}",
            n
        );
        track_assert!(
            n <= std::i32::MAX as i64,
            ErrorKind::Invalid,
            "Tool large `int32` value: {}",
            n
        );
        Ok(Sint32(n as i32))
    }
}

#[derive(Debug, Default)]
pub struct Sint64(pub i64);
impl_scalar_type!(Sint64, i64, Varint);
impl Packable for Sint64 {}
impl From<Varint> for Sint64 {
    fn from(f: Varint) -> Self {
        let n = ((f.0 << 63) | (f.0 >> 1)) as i64;
        Sint64(n)
    }
}

#[derive(Debug, Default)]
pub struct Fixed32(pub u32);
impl_scalar_type!(Fixed32, u32, Bit32);
impl Packable for Fixed32 {}
impl From<Bit32> for Fixed32 {
    fn from(f: Bit32) -> Self {
        Fixed32(LittleEndian::read_u32(&f.0[..]))
    }
}

#[derive(Debug, Default)]
pub struct Fixed64(pub u64);
impl_scalar_type!(Fixed64, u64, Bit64);
impl Packable for Fixed64 {}
impl From<Bit64> for Fixed64 {
    fn from(f: Bit64) -> Self {
        Fixed64(LittleEndian::read_u64(&f.0[..]))
    }
}

#[derive(Debug, Default)]
pub struct Sfixed32(pub i32);
impl_scalar_type!(Sfixed32, i32, Bit32);
impl Packable for Sfixed32 {}
impl From<Bit32> for Sfixed32 {
    fn from(f: Bit32) -> Self {
        Sfixed32(LittleEndian::read_i32(&f.0[..]))
    }
}

#[derive(Debug, Default)]
pub struct Sfixed64(pub i64);
impl_scalar_type!(Sfixed64, i64, Bit64);
impl Packable for Sfixed64 {}
impl From<Bit64> for Sfixed64 {
    fn from(f: Bit64) -> Self {
        Sfixed64(LittleEndian::read_i64(&f.0[..]))
    }
}

#[derive(Debug, Default)]
pub struct Float(pub f32);
impl_scalar_type!(Float, f32, Bit32);
impl Packable for Float {}
impl From<Bit32> for Float {
    fn from(f: Bit32) -> Self {
        Float(LittleEndian::read_f32(&f.0[..]))
    }
}

#[derive(Debug, Default)]
pub struct Double(pub f64);
impl_scalar_type!(Double, f64, Bit64);
impl Packable for Double {}
impl From<Bit64> for Double {
    fn from(f: Bit64) -> Self {
        Double(LittleEndian::read_f64(&f.0[..]))
    }
}

#[derive(Debug, Default)]
pub struct Bytes(pub Vec<u8>);
impl_scalar_type!(Bytes, Vec<u8>, LengthDelimited);
impl From<LengthDelimited<Vec<u8>>> for Bytes {
    fn from(f: LengthDelimited<Vec<u8>>) -> Self {
        Bytes(f.0)
    }
}

#[derive(Debug, Default)]
pub struct Str(pub String);
impl_scalar_type!(Str, String, LengthDelimited);
impl TryFrom<Bytes> for Str {
    fn try_from(f: Bytes) -> Result<Str> {
        String::from_utf8(f.0).map(Str).map_err(|e| {
            ErrorKind::Invalid.cause(e)
        })
    }
}

#[derive(Debug, Default)]
pub struct Embedded<T: Message>(pub T);
impl<T: Message> FieldType for Embedded<T> {
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}
impl<T: Message> From<T> for Embedded<T> {
    fn from(f: T) -> Self {
        Embedded(f)
    }
}
