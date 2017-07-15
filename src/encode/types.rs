use std::io::Write;
use byteorder::{ByteOrder, LittleEndian};
use futures::{Future, Poll, Async};

use Error;
use types;
use util_futures::Phase2;
use wire::types::{Varint, LengthDelimited};
use super::Encode;
use super::futures::{EncodeVarint, EncodeLengthDelimited, WriteBytes};

impl<W: Write> Encode<W> for types::Bool {
    type Value = bool;
    type Future = EncodeVarint<W>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        Varint::encode(writer, value as u64)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        1
    }
}
impl<W: Write> Encode<W> for types::Uint32 {
    type Value = u32;
    type Future = EncodeVarint<W>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        Varint::encode(writer, value as u64)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let n = *value as u64;
        <Varint as Encode<W>>::encoded_size(&n)
    }
}
impl<W: Write> Encode<W> for types::Uint64 {
    type Value = u64;
    type Future = EncodeVarint<W>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        Varint::encode(writer, value)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        <Varint as Encode<W>>::encoded_size(value)
    }
}
impl<W: Write> Encode<W> for types::Int32 {
    type Value = i32;
    type Future = EncodeVarint<W>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        Varint::encode(writer, value as u64)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let n = *value as u64;
        <Varint as Encode<W>>::encoded_size(&n)
    }
}
impl<W: Write> Encode<W> for types::Int64 {
    type Value = i64;
    type Future = EncodeVarint<W>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        Varint::encode(writer, value as u64)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let n = *value as u64;
        <Varint as Encode<W>>::encoded_size(&n)
    }
}
impl<W: Write> Encode<W> for types::Sint32 {
    type Value = i32;
    type Future = EncodeVarint<W>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        let n = value as u32;
        let n = (n << 1) | (n >> 31);
        Varint::encode(writer, n as u64)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let n = *value as u32;
        let n = ((n << 1) | (n >> 31)) as u64;
        <Varint as Encode<W>>::encoded_size(&n)
    }
}
impl<W: Write> Encode<W> for types::Sint64 {
    type Value = i64;
    type Future = EncodeVarint<W>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        let n = value as u64;
        let n = (n << 1) | (n >> 63);
        Varint::encode(writer, n)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let n = *value as u64;
        let n = (n << 1) | (n >> 63);
        <Varint as Encode<W>>::encoded_size(&n)
    }
}
impl<W: Write> Encode<W> for types::Fixed32 {
    type Value = u32;
    type Future = WriteBytes<W, [u8; 4]>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        let mut bytes = [0; 4];
        LittleEndian::write_u32(&mut bytes, value);
        WriteBytes::new(writer, bytes)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        4
    }
}
impl<W: Write> Encode<W> for types::Fixed64 {
    type Value = u64;
    type Future = WriteBytes<W, [u8; 8]>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        let mut bytes = [0; 8];
        LittleEndian::write_u64(&mut bytes, value);
        WriteBytes::new(writer, bytes)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        8
    }
}
impl<W: Write> Encode<W> for types::Sfixed32 {
    type Value = i32;
    type Future = WriteBytes<W, [u8; 4]>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        let mut bytes = [0; 4];
        LittleEndian::write_i32(&mut bytes, value);
        WriteBytes::new(writer, bytes)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        4
    }
}
impl<W: Write> Encode<W> for types::Sfixed64 {
    type Value = i64;
    type Future = WriteBytes<W, [u8; 8]>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        let mut bytes = [0; 8];
        LittleEndian::write_i64(&mut bytes, value);
        WriteBytes::new(writer, bytes)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        8
    }
}
impl<W: Write> Encode<W> for types::Bytes {
    type Value = Vec<u8>;
    type Future = EncodeLengthDelimited<W, Vec<u8>>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        LengthDelimited::<Vec<u8>>::encode(writer, value)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        value.len() as u64
    }
}
impl<W: Write> Encode<W> for types::Str {
    type Value = String;
    type Future = EncodeLengthDelimited<W, Vec<u8>>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        LengthDelimited::<Vec<u8>>::encode(writer, value.into_bytes())
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        value.as_bytes().len() as u64
    }
}

#[derive(Debug)]
pub struct EncodeMessage2<W, A, B>
where
    W: Write,
    A: Encode<W>,
    B: Encode<W>,
{
    phase: Phase2<A::Future, B::Future>,
    value1: Option<B::Value>,
}
impl<W, A, B> EncodeMessage2<W, A, B>
where
    W: Write,
    A: Encode<W>,
    B: Encode<W>,
{
    fn new(writer: W, a: A::Value, b: B::Value) -> Self {
        let phase = Phase2::A(A::encode(writer, a));
        EncodeMessage2 {
            phase,
            value1: Some(b),
        }
    }
}
impl<W, A, B> Future for EncodeMessage2<W, A, B>
where
    W: Write,
    A: Encode<W>,
    B: Encode<W>,
{
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let next = match phase {
                Phase2::A(w) => Phase2::B(B::encode(w, self.value1.take().expect("Never fails"))),
                Phase2::B(w) => return Ok(Async::Ready(w)),
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}
impl<W: Write, A, B> Encode<W> for types::Message<(A, B)>
where
    A: Encode<W>,
    B: Encode<W>,
{
    type Value = (A::Value, B::Value);
    type Future = EncodeMessage2<W, A, B>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        EncodeMessage2::new(writer, value.0, value.1)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        A::encoded_size(&value.0) + B::encoded_size(&value.1)
    }
}
