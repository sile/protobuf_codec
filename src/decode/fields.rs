use std::io::Read;
use trackable::error::ErrorKindExt;

use {Tag, WireType};
use decode::{Error, ErrorKind, FieldDecode, Decode, FieldDecodeResult};
use decode::futures;
use decode::types;

macro_rules! assert_wire_type {
    ($actual:expr, $expected:expr, $reader:expr) => {
        if $actual != $expected {
            let cause = format!("Unexpected wire type: actual={:?}, expected={:?}", $actual, $expected);
            return FieldDecodeResult::Err(track!(Error::new($reader, ErrorKind::Invalid.cause(cause))));
        }
    }
}

macro_rules! try_field_decode {
    ($expr:expr) => {
        match $expr {
            FieldDecodeResult::Ok(v) => return FieldDecodeResult::Ok(v),
            FieldDecodeResult::Err(e) => return FieldDecodeResult::Err(track!(e)),
            FieldDecodeResult::NotSubject(r) => r,
        }
    }
}

impl<R: Read> FieldDecode<R> for types::Uint32 {
    type Value = u32;
    type Future = futures::DecodeUint32<R>;
    fn field_decode(
        &self,
        _tag: Tag,
        wire_type: WireType,
        reader: R,
    ) -> FieldDecodeResult<Self::Future, R> {
        assert_wire_type!(wire_type, WireType::Varint, reader);
        FieldDecodeResult::Ok(self.decode(reader))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Field<T> {
    pub tag: Tag,
    pub name: &'static str,
    pub value: T,
}
impl<R: Read, T: FieldDecode<R>> FieldDecode<R> for Field<T> {
    type Value = T::Value;
    type Future = T::Future;
    fn field_decode(
        &self,
        tag: Tag,
        wire_type: WireType,
        reader: R,
    ) -> FieldDecodeResult<Self::Future, R> {
        if tag != self.tag {
            FieldDecodeResult::NotSubject(reader)
        } else {
            // TODO: track
            self.value.field_decode(tag, wire_type, reader)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Oneof2<F0, F1> {
    pub name: &'static str,
    pub field0: F0,
    pub field1: F1,
}
impl<R: Read, F0, F1> FieldDecode<R> for Oneof2<F0, F1>
where
    F0: FieldDecode<R>,
    F1: FieldDecode<R>,
{
    type Value = Variant2<F0::Value, F1::Value>;
    type Future = futures::DecodeVariant2<R, F0, F1>;
    fn field_decode(
        &self,
        tag: Tag,
        wire_type: WireType,
        reader: R,
    ) -> FieldDecodeResult<Self::Future, R> {
        let reader = try_field_decode!(self.field0.field_decode(tag, wire_type, reader).map(
            futures::DecodeVariant2::A,
        ));
        let reader = try_field_decode!(self.field1.field_decode(tag, wire_type, reader).map(
            futures::DecodeVariant2::B,
        ));
        FieldDecodeResult::NotSubject(reader)
    }
}

#[derive(Debug)]
pub enum Variant2<A, B> {
    A(A),
    B(B),
    None,
}
impl<A, B> Default for Variant2<A, B> {
    fn default() -> Self {
        Variant2::None
    }
}
