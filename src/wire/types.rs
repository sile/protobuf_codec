//! Wire Value Types.
//!
//! https://developers.google.com/protocol-buffers/docs/encoding#structure
use std::marker::PhantomData;

use traits::Pattern;

pub struct Varint;
impl Pattern for Varint {
    type Value = u64;
}

pub struct Bit32;
impl Pattern for Bit32 {
    type Value = [u8; 4];
}

pub struct Bit64;
impl Pattern for Bit64 {
    type Value = [u8; 8];
}

pub struct LengthDelimited<T>(PhantomData<T>);
impl<T: Pattern> Pattern for LengthDelimited<T> {
    type Value = T::Value;
}
