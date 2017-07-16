use std::io::Read;

use Decode;
use future::decode::{DecodeInto, DecodeTryInto};
use types;
use wire::types::{Varint, Bit32, Bit64, LengthDelimited};

impl<R: Read> Decode<R> for types::Bool {
    type Future = DecodeTryInto<R, Varint, Self>;
    fn decode(reader: R) -> Self::Future {
        Varint::decode_try_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Uint32 {
    type Future = DecodeTryInto<R, Varint, Self>;
    fn decode(reader: R) -> Self::Future {
        Varint::decode_try_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Uint64 {
    type Future = DecodeInto<R, Varint, Self>;
    fn decode(reader: R) -> Self::Future {
        Varint::decode_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Int32 {
    type Future = DecodeTryInto<R, Varint, Self>;
    fn decode(reader: R) -> Self::Future {
        Varint::decode_try_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Int64 {
    type Future = DecodeInto<R, Varint, Self>;
    fn decode(reader: R) -> Self::Future {
        Varint::decode_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Sint32 {
    type Future = DecodeTryInto<R, Varint, Self>;
    fn decode(reader: R) -> Self::Future {
        Varint::decode_try_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Sint64 {
    type Future = DecodeInto<R, Varint, Self>;
    fn decode(reader: R) -> Self::Future {
        Varint::decode_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Fixed32 {
    type Future = DecodeInto<R, Bit32, Self>;
    fn decode(reader: R) -> Self::Future {
        Bit32::decode_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Fixed64 {
    type Future = DecodeInto<R, Bit64, Self>;
    fn decode(reader: R) -> Self::Future {
        Bit64::decode_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Sfixed32 {
    type Future = DecodeInto<R, Bit32, Self>;
    fn decode(reader: R) -> Self::Future {
        Bit32::decode_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Sfixed64 {
    type Future = DecodeInto<R, Bit64, Self>;
    fn decode(reader: R) -> Self::Future {
        Bit64::decode_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Float {
    type Future = DecodeInto<R, Bit32, Self>;
    fn decode(reader: R) -> Self::Future {
        Bit32::decode_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Double {
    type Future = DecodeInto<R, Bit64, Self>;
    fn decode(reader: R) -> Self::Future {
        Bit64::decode_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Bytes {
    type Future = DecodeInto<R, LengthDelimited<Vec<u8>>, Self>;
    fn decode(reader: R) -> Self::Future {
        Decode::decode_into(reader)
    }
}

impl<R: Read> Decode<R> for types::Str {
    type Future = DecodeTryInto<R, types::Bytes, Self>;
    fn decode(reader: R) -> Self::Future {
        Decode::decode_try_into(reader)
    }
}
