//! Wire Value Types.
//!
//! https://developers.google.com/protocol-buffers/docs/encoding#structure
use std::marker::PhantomData;

pub struct Varint;
pub struct Bit32;
pub struct Bit64;
pub struct LengthDelimited<T>(PhantomData<T>);
