use traits::{Type, Field};
use wire::WireType;

macro_rules! impl_scalar_type {
    ($t:ident, $base:ty, $wire:ident) => {
        impl Type for $t {
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

#[derive(Debug, Default)]
pub struct Int32(pub i32);
impl_scalar_type!(Int32, i32, Varint);

#[derive(Debug, Default)]
pub struct Int64(pub i64);
impl_scalar_type!(Int64, i64, Varint);

#[derive(Debug, Default)]
pub struct Uint32(pub u32);
impl_scalar_type!(Uint32, u32, Varint);

#[derive(Debug, Default)]
pub struct Uint64(pub u64);
impl_scalar_type!(Uint64, u64, Varint);

#[derive(Debug, Default)]
pub struct Sint32(pub i32);
impl_scalar_type!(Sint32, i32, Varint);

#[derive(Debug, Default)]
pub struct Sint64(pub i64);
impl_scalar_type!(Sint64, i64, Varint);

#[derive(Debug, Default)]
pub struct Fixed32(pub u32);
impl_scalar_type!(Fixed32, u32, Bit32);

#[derive(Debug, Default)]
pub struct Fixed64(pub u64);
impl_scalar_type!(Fixed64, u64, Bit64);

#[derive(Debug, Default)]
pub struct Sfixed32(pub i32);
impl_scalar_type!(Sfixed32, i32, Bit32);

#[derive(Debug, Default)]
pub struct Sfixed64(pub i64);
impl_scalar_type!(Sfixed64, i64, Bit64);

#[derive(Debug, Default)]
pub struct Bytes(pub Vec<u8>);
impl_scalar_type!(Bytes, Vec<u8>, LengthDelimited);

#[derive(Debug, Default)]
pub struct Str(pub String);
impl_scalar_type!(Str, String, LengthDelimited);

#[derive(Debug, Default)]
pub struct Message<Fields> {
    pub fields: Fields,
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
