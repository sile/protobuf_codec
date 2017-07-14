use std::io::Write;
use byteorder::{ByteOrder, LittleEndian};

use scalars;
use wires;
use super::Encode;
use super::futures::{EncodeVarint, EncodeLengthDelimited, WriteBytes};

impl<W: Write> Encode<W> for scalars::Bool {
    type Value = bool;
    type Future = EncodeVarint<W>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        wires::Varint::encode(value as u64, writer)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        1
    }
}
impl<W: Write> Encode<W> for scalars::Uint32 {
    type Value = u32;
    type Future = EncodeVarint<W>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        wires::Varint::encode(value as u64, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let n = *value as u64;
        <wires::Varint as Encode<W>>::encoded_size(&n)
    }
}
impl<W: Write> Encode<W> for scalars::Uint64 {
    type Value = u64;
    type Future = EncodeVarint<W>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        wires::Varint::encode(value, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        <wires::Varint as Encode<W>>::encoded_size(value)
    }
}
impl<W: Write> Encode<W> for scalars::Int32 {
    type Value = i32;
    type Future = EncodeVarint<W>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        wires::Varint::encode(value as u64, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let n = *value as u64;
        <wires::Varint as Encode<W>>::encoded_size(&n)
    }
}
impl<W: Write> Encode<W> for scalars::Int64 {
    type Value = i64;
    type Future = EncodeVarint<W>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        wires::Varint::encode(value as u64, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let n = *value as u64;
        <wires::Varint as Encode<W>>::encoded_size(&n)
    }
}
impl<W: Write> Encode<W> for scalars::Sint32 {
    type Value = i32;
    type Future = EncodeVarint<W>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        let n = value as u32;
        let n = (n << 1) | (n >> 31);
        wires::Varint::encode(n as u64, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let n = *value as u32;
        let n = ((n << 1) | (n >> 31)) as u64;
        <wires::Varint as Encode<W>>::encoded_size(&n)
    }
}
impl<W: Write> Encode<W> for scalars::Sint64 {
    type Value = i64;
    type Future = EncodeVarint<W>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        let n = value as u64;
        let n = (n << 1) | (n >> 63);
        wires::Varint::encode(n, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let n = *value as u64;
        let n = (n << 1) | (n >> 63);
        <wires::Varint as Encode<W>>::encoded_size(&n)
    }
}
impl<W: Write> Encode<W> for scalars::Fixed32 {
    type Value = u32;
    type Future = WriteBytes<W, [u8; 4]>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        let mut bytes = [0; 4];
        LittleEndian::write_u32(&mut bytes, value);
        WriteBytes::new(bytes, writer)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        4
    }
}
impl<W: Write> Encode<W> for scalars::Fixed64 {
    type Value = u64;
    type Future = WriteBytes<W, [u8; 8]>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        let mut bytes = [0; 8];
        LittleEndian::write_u64(&mut bytes, value);
        WriteBytes::new(bytes, writer)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        8
    }
}
impl<W: Write> Encode<W> for scalars::Sfixed32 {
    type Value = i32;
    type Future = WriteBytes<W, [u8; 4]>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        let mut bytes = [0; 4];
        LittleEndian::write_i32(&mut bytes, value);
        WriteBytes::new(bytes, writer)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        4
    }
}
impl<W: Write> Encode<W> for scalars::Sfixed64 {
    type Value = i64;
    type Future = WriteBytes<W, [u8; 8]>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        let mut bytes = [0; 8];
        LittleEndian::write_i64(&mut bytes, value);
        WriteBytes::new(bytes, writer)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        8
    }
}
impl<W: Write> Encode<W> for scalars::Bytes {
    type Value = Vec<u8>;
    type Future = EncodeLengthDelimited<W, Vec<u8>>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        wires::LengthDelimited::<Vec<u8>>::encode(value, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        value.len() as u64
    }
}
impl<W: Write> Encode<W> for scalars::Str {
    type Value = String;
    type Future = EncodeLengthDelimited<W, Vec<u8>>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        wires::LengthDelimited::<Vec<u8>>::encode(value.into_bytes(), writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        value.as_bytes().len() as u64
    }
}
