//! Encoders and decoders and related components for [scalar] values.
//!
//! [scalar]: https://developers.google.com/protocol-buffers/docs/proto3#scalar
use bytecodec::{ByteCount, Decode, Encode, Eos, ExactBytesEncode, Result};
use bytecodec::bytes::{BytesEncoder as BytesEncoderInner, RemainingBytesDecoder, Utf8Decoder,
                       Utf8Encoder};
use bytecodec::fixnum::{F32leDecoder, F32leEncoder, F64leDecoder, F64leEncoder, I32leDecoder,
                        I32leEncoder, I64leDecoder, I64leEncoder, U32leDecoder, U32leEncoder,
                        U64leDecoder, U64leEncoder};

use wire::{LengthDelimitedDecoder, LengthDelimitedEncoder, VarintDecoder, VarintEncoder,
           WireDecode, WireEncode, WireType};

pub trait MapKeyDecode: WireDecode {}

pub trait MapKeyEncode: WireEncode {}

pub trait NumericDecode: WireDecode {}

pub trait NumericEncode: WireEncode {}

macro_rules! impl_newtype_decode {
    ($decoder:ty, $item:ty, $wire:ident) => {
        impl Decode for $decoder {
            type Item = $item;

            fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
                track!(self.0.decode(buf, eos))
            }

            fn requiring_bytes(&self) -> ByteCount {
                self.0.requiring_bytes()
            }
        }
        impl WireDecode for $decoder {
            type Value = $item;

            fn wire_type(&self) -> WireType {
                WireType::$wire
            }

            fn merge(&self, _old: Self::Value, new: Self::Item) -> Self::Value {
                new
            }
        }
    }
}

macro_rules! impl_newtype_encode {
    ($encoder:ty, $item:ty, $wire:ident) => {
        impl Encode for $encoder {
            type Item = $item;

            fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
                track!(self.0.encode(buf, eos))
            }

            fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
                track!(self.0.start_encoding(item))
            }

            fn is_idle(&self) -> bool {
                self.0.is_idle()
            }

            fn requiring_bytes(&self) -> ByteCount {
                self.0.requiring_bytes()
            }
        }
        impl ExactBytesEncode for $encoder {
            fn exact_requiring_bytes(&self) -> u64 {
                self.0.exact_requiring_bytes()
            }
        }
        impl WireEncode for $encoder {
            type Value = $item;

            fn wire_type(&self) -> WireType {
                WireType::$wire
            }

            fn start_encoding_value(&mut self, value: Self::Value) -> Result<()> {
                if value != Default::default() {
                    track!(self.start_encoding(value))?;
                }
                Ok(())
            }
        }
    }
}

macro_rules! impl_varint_decode {
    ($decoder:ty, $item:ty) => {
        impl Decode for $decoder {
            type Item = $item;

            fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
                let (size, item) = track!(self.0.decode(buf, eos))?;
                let item = item.map(Self::from_varint);
                Ok((size, item))
            }

            fn requiring_bytes(&self) -> ByteCount {
                self.0.requiring_bytes()
            }
        }
        impl WireDecode for $decoder {
            type Value = $item;

            fn wire_type(&self) -> WireType {
                WireType::Varint
            }

            fn merge(&self, _old: Self::Value, new: Self::Item) -> Self::Value {
                new
            }
        }
    }
}

macro_rules! impl_varint_encode {
    ($encoder:ty, $item:ty) => {
        impl Encode for $encoder {
            type Item = $item;

            fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
                track!(self.0.encode(buf, eos))
            }

            fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
                track!(self.0.start_encoding(Self::to_varint(item)))
            }

            fn is_idle(&self) -> bool {
                self.0.is_idle()
            }

            fn requiring_bytes(&self) -> ByteCount {
                self.0.requiring_bytes()
            }
        }
        impl ExactBytesEncode for $encoder {
            fn exact_requiring_bytes(&self) -> u64 {
                self.0.exact_requiring_bytes()
            }
        }
        impl WireEncode for $encoder {
            type Value = $item;

            fn wire_type(&self) -> WireType {
                WireType::Varint
            }

            fn start_encoding_value(&mut self, value: Self::Value) -> Result<()> {
                if value != Default::default() {
                    track!(self.start_encoding(value))?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct DoubleDecoder(F64leDecoder);
impl DoubleDecoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(DoubleDecoder, f64, Bit64);
impl NumericDecode for DoubleDecoder {}

#[derive(Debug, Default)]
pub struct DoubleEncoder(F64leEncoder);
impl DoubleEncoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(DoubleEncoder, f64, Bit64);
impl NumericEncode for DoubleEncoder {}

#[derive(Debug, Default)]
pub struct FloatDecoder(F32leDecoder);
impl FloatDecoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(FloatDecoder, f32, Bit32);
impl NumericDecode for FloatDecoder {}

#[derive(Debug, Default)]
pub struct FloatEncoder(F32leEncoder);
impl FloatEncoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(FloatEncoder, f32, Bit32);
impl NumericEncode for FloatEncoder {}

#[derive(Debug, Default)]
pub struct Fixed32Decoder(U32leDecoder);
impl Fixed32Decoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(Fixed32Decoder, u32, Bit32);
impl MapKeyDecode for Fixed32Decoder {}
impl NumericDecode for Fixed32Decoder {}

#[derive(Debug, Default)]
pub struct Fixed32Encoder(U32leEncoder);
impl Fixed32Encoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(Fixed32Encoder, u32, Bit32);
impl MapKeyEncode for Fixed32Encoder {}
impl NumericEncode for Fixed32Encoder {}

#[derive(Debug, Default)]
pub struct Fixed64Decoder(U64leDecoder);
impl Fixed64Decoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(Fixed64Decoder, u64, Bit64);
impl MapKeyDecode for Fixed64Decoder {}
impl NumericDecode for Fixed64Decoder {}

#[derive(Debug, Default)]
pub struct Fixed64Encoder(U64leEncoder);
impl Fixed64Encoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(Fixed64Encoder, u64, Bit64);
impl MapKeyEncode for Fixed64Encoder {}
impl NumericEncode for Fixed64Encoder {}

#[derive(Debug, Default)]
pub struct Sfixed32Decoder(I32leDecoder);
impl Sfixed32Decoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(Sfixed32Decoder, i32, Bit32);
impl MapKeyDecode for Sfixed32Decoder {}
impl NumericDecode for Sfixed32Decoder {}

#[derive(Debug, Default)]
pub struct Sfixed32Encoder(I32leEncoder);
impl Sfixed32Encoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(Sfixed32Encoder, i32, Bit32);
impl MapKeyEncode for Sfixed32Encoder {}
impl NumericEncode for Sfixed32Encoder {}

#[derive(Debug, Default)]
pub struct Sfixed64Decoder(I64leDecoder);
impl Sfixed64Decoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(Sfixed64Decoder, i64, Bit64);
impl MapKeyDecode for Sfixed64Decoder {}
impl NumericDecode for Sfixed64Decoder {}

#[derive(Debug, Default)]
pub struct Sfixed64Encoder(I64leEncoder);
impl Sfixed64Encoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(Sfixed64Encoder, i64, Bit64);
impl MapKeyEncode for Sfixed64Encoder {}
impl NumericEncode for Sfixed64Encoder {}

#[derive(Debug, Default)]
pub struct BoolDecoder(VarintDecoder);
impl BoolDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn from_varint(n: u64) -> bool {
        n != 0
    }
}
impl_varint_decode!(BoolDecoder, bool);
impl MapKeyDecode for BoolDecoder {}
impl NumericDecode for BoolDecoder {}

#[derive(Debug, Default)]
pub struct BoolEncoder(VarintEncoder);
impl BoolEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn to_varint(n: bool) -> u64 {
        n as u64
    }
}
impl_varint_encode!(BoolEncoder, bool);
impl MapKeyEncode for BoolEncoder {}
impl NumericEncode for BoolEncoder {}

#[derive(Debug, Default)]
pub struct Int32Decoder(VarintDecoder);
impl Int32Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn from_varint(n: u64) -> i32 {
        n as i32
    }
}
impl_varint_decode!(Int32Decoder, i32);
impl MapKeyDecode for Int32Decoder {}
impl NumericDecode for Int32Decoder {}

#[derive(Debug, Default)]
pub struct Int32Encoder(VarintEncoder);
impl Int32Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn to_varint(n: i32) -> u64 {
        n as u64
    }
}
impl_varint_encode!(Int32Encoder, i32);
impl MapKeyEncode for Int32Encoder {}
impl NumericEncode for Int32Encoder {}

#[derive(Debug, Default)]
pub struct Int64Decoder(VarintDecoder);
impl Int64Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn from_varint(n: u64) -> i64 {
        n as i64
    }
}
impl_varint_decode!(Int64Decoder, i64);
impl MapKeyDecode for Int64Decoder {}
impl NumericDecode for Int64Decoder {}

#[derive(Debug, Default)]
pub struct Int64Encoder(VarintEncoder);
impl Int64Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn to_varint(n: i64) -> u64 {
        n as u64
    }
}
impl_varint_encode!(Int64Encoder, i64);
impl MapKeyEncode for Int64Encoder {}
impl NumericEncode for Int64Encoder {}

#[derive(Debug, Default)]
pub struct Uint32Decoder(VarintDecoder);
impl Uint32Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn from_varint(n: u64) -> u32 {
        n as u32
    }
}
impl_varint_decode!(Uint32Decoder, u32);
impl MapKeyDecode for Uint32Decoder {}
impl NumericDecode for Uint32Decoder {}

#[derive(Debug, Default)]
pub struct Uint32Encoder(VarintEncoder);
impl Uint32Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn to_varint(n: u32) -> u64 {
        n as u64
    }
}
impl_varint_encode!(Uint32Encoder, u32);
impl MapKeyEncode for Uint32Encoder {}
impl NumericEncode for Uint32Encoder {}

#[derive(Debug, Default)]
pub struct Uint64Decoder(VarintDecoder);
impl Uint64Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn from_varint(n: u64) -> u64 {
        n
    }
}
impl_varint_decode!(Uint64Decoder, u64);
impl MapKeyDecode for Uint64Decoder {}
impl NumericDecode for Uint64Decoder {}

#[derive(Debug, Default)]
pub struct Uint64Encoder(VarintEncoder);
impl Uint64Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn to_varint(n: u64) -> u64 {
        n
    }
}
impl_varint_encode!(Uint64Encoder, u64);
impl MapKeyEncode for Uint64Encoder {}
impl NumericEncode for Uint64Encoder {}

#[derive(Debug, Default)]
pub struct Sint32Decoder(VarintDecoder);
impl Sint32Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn from_varint(n: u64) -> i32 {
        let n = n as i32;
        (n >> 1) ^ ((n << 31) >> 31)
    }
}
impl_varint_decode!(Sint32Decoder, i32);
impl MapKeyDecode for Sint32Decoder {}
impl NumericDecode for Sint32Decoder {}

#[derive(Debug, Default)]
pub struct Sint32Encoder(VarintEncoder);
impl Sint32Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn to_varint(n: i32) -> u64 {
        ((n << 1) ^ (n >> 31)) as u64
    }
}
impl_varint_encode!(Sint32Encoder, i32);
impl MapKeyEncode for Sint32Encoder {}
impl NumericEncode for Sint32Encoder {}

#[derive(Debug, Default)]
pub struct Sint64Decoder(VarintDecoder);
impl Sint64Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn from_varint(n: u64) -> i64 {
        let n = n as i64;
        (n >> 1) ^ ((n << 63) >> 63)
    }
}
impl_varint_decode!(Sint64Decoder, i64);
impl MapKeyDecode for Sint64Decoder {}
impl NumericDecode for Sint64Decoder {}

#[derive(Debug, Default)]
pub struct Sint64Encoder(VarintEncoder);
impl Sint64Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn to_varint(n: i64) -> u64 {
        ((n << 1) ^ (n >> 63)) as u64
    }
}
impl_varint_encode!(Sint64Encoder, i64);
impl MapKeyEncode for Sint64Encoder {}
impl NumericEncode for Sint64Encoder {}

#[derive(Debug, Default)]
pub struct BytesDecoder(LengthDelimitedDecoder<RemainingBytesDecoder>);
impl BytesDecoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(BytesDecoder, Vec<u8>, LengthDelimited);

// TODO: CustomBytesEncoder

#[derive(Debug)]
pub struct BytesEncoder<B = Vec<u8>>(LengthDelimitedEncoder<BytesEncoderInner<B>>);
impl<B> BytesEncoder<B> {
    pub fn new() -> Self {
        Self::default()
    }
}
impl<B> Default for BytesEncoder<B> {
    fn default() -> Self {
        BytesEncoder(Default::default())
    }
}
impl<B: AsRef<[u8]>> Encode for BytesEncoder<B> {
    type Item = B;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.0.start_encoding(item))
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<B: AsRef<[u8]>> ExactBytesEncode for BytesEncoder<B> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}
impl<B: AsRef<[u8]>> WireEncode for BytesEncoder<B> {
    type Value = B;

    fn wire_type(&self) -> WireType {
        WireType::LengthDelimited
    }

    fn start_encoding_value(&mut self, value: Self::Value) -> Result<()> {
        if !value.as_ref().is_empty() {
            track!(self.start_encoding(value))?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct StringDecoder(LengthDelimitedDecoder<Utf8Decoder>);
impl StringDecoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(StringDecoder, String, LengthDelimited);
impl MapKeyDecode for StringDecoder {}

#[derive(Debug)]
pub struct StringEncoder<S = String>(LengthDelimitedEncoder<Utf8Encoder<S>>);
impl<S> StringEncoder<S> {
    pub fn new() -> Self {
        Self::default()
    }
}
impl<S> Default for StringEncoder<S> {
    fn default() -> Self {
        StringEncoder(Default::default())
    }
}
impl<S: AsRef<str>> Encode for StringEncoder<S> {
    type Item = S;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.0.start_encoding(item))
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<S: AsRef<str>> ExactBytesEncode for StringEncoder<S> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}
impl<S: AsRef<str>> WireEncode for StringEncoder<S> {
    type Value = S;

    fn wire_type(&self) -> WireType {
        WireType::LengthDelimited
    }

    fn start_encoding_value(&mut self, value: Self::Value) -> Result<()> {
        if !value.as_ref().is_empty() {
            track!(self.start_encoding(value))?;
        }
        Ok(())
    }
}
impl<S: AsRef<str>> MapKeyEncode for StringEncoder<S> {}

#[cfg(test)]
mod test {
    use bytecodec::EncodeExt;
    use bytecodec::io::{IoDecodeExt, IoEncodeExt};

    use super::*;

    macro_rules! assert_decode {
        ($decoder:ident, $value:expr, $bytes:expr) => {
            let mut decoder = $decoder::new();
            let item = track_try_unwrap!(decoder.decode_exact($bytes.as_ref()));
            assert_eq!(item, $value);
        }
    }

    macro_rules! assert_encode {
        ($encoder:ident, $value:expr, $bytes:expr) => {
            let mut buf = Vec::new();
            let mut encoder = track_try_unwrap!($encoder::with_item($value));
            track_try_unwrap!(encoder.encode_all(&mut buf));
            assert_eq!(buf, $bytes);
        }
    }

    #[test]
    fn double_decoder_works() {
        assert_decode!(
            DoubleDecoder,
            1.23,
            [0xae, 0x47, 0xe1, 0x7a, 0x14, 0xae, 0xf3, 0x3f]
        );
    }

    #[test]
    fn double_encoder_works() {
        assert_encode!(
            DoubleEncoder,
            1.23,
            [0xae, 0x47, 0xe1, 0x7a, 0x14, 0xae, 0xf3, 0x3f]
        );
    }

    #[test]
    fn float_decoder_works() {
        assert_decode!(FloatDecoder, 3.25, [0x00, 0x00, 0x50, 0x40]);
    }

    #[test]
    fn float_encoder_works() {
        assert_encode!(FloatEncoder, 3.25, [0x00, 0x00, 0x50, 0x40]);
    }

    #[test]
    fn fixed32_decoder_works() {
        assert_decode!(Fixed32Decoder, 12345678, [0x4e, 0x61, 0xbc, 0x00]);
    }

    #[test]
    fn fixed32_encoder_works() {
        assert_encode!(Fixed32Encoder, 12345678, [0x4e, 0x61, 0xbc, 0x00]);
    }

    #[test]
    fn fixed64_decoder_works() {
        assert_decode!(
            Fixed64Decoder,
            1234567890987654321,
            [0xb1, 0x1c, 0x6c, 0xb1, 0xf4, 0x10, 0x22, 0x11]
        );
    }

    #[test]
    fn fixed64_encoder_works() {
        assert_encode!(
            Fixed64Encoder,
            1234567890987654321,
            [0xb1, 0x1c, 0x6c, 0xb1, 0xf4, 0x10, 0x22, 0x11]
        );
    }

    #[test]
    fn sfixed32_decoder_works() {
        assert_decode!(Sfixed32Decoder, -123456789, [0xeb, 0x32, 0xa4, 0xf8]);
    }

    #[test]
    fn sfixed32_encoder_works() {
        assert_encode!(Sfixed32Encoder, -123456789, [0xeb, 0x32, 0xa4, 0xf8]);
    }

    #[test]
    fn sfixed64_decoder_works() {
        assert_decode!(
            Sfixed64Decoder,
            -1234567890987654321,
            [0x4f, 0xe3, 0x93, 0x4e, 0x0b, 0xef, 0xdd, 0xee]
        );
    }

    #[test]
    fn sfixed64_encoder_works() {
        assert_encode!(
            Sfixed64Encoder,
            -1234567890987654321,
            [0x4f, 0xe3, 0x93, 0x4e, 0x0b, 0xef, 0xdd, 0xee]
        );
    }

    #[test]
    fn bool_decoder_works() {
        assert_decode!(BoolDecoder, false, [0x00]);
        assert_decode!(BoolDecoder, true, [0x01]);
        assert_decode!(BoolDecoder, true, [0xFF, 0xFF, 0x01]);
    }

    #[test]
    fn bool_encoder_works() {
        assert_encode!(BoolEncoder, false, [0x00]);
        assert_encode!(BoolEncoder, true, [0x01]);
    }

    #[test]
    fn int32_decoder_works() {
        assert_decode!(
            Int32Decoder,
            -12345678,
            [0xb2, 0xbd, 0x8e, 0xfa, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]
        );
    }

    #[test]
    fn int32_encoder_works() {
        assert_encode!(
            Int32Encoder,
            -12345678,
            [0xb2, 0xbd, 0x8e, 0xfa, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]
        );
    }

    #[test]
    fn int64_decoder_works() {
        assert_decode!(
            Int64Decoder,
            -12345678,
            [0xb2, 0xbd, 0x8e, 0xfa, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]
        );
    }

    #[test]
    fn int64_encoder_works() {
        assert_encode!(
            Int64Encoder,
            -12345678,
            [0xb2, 0xbd, 0x8e, 0xfa, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]
        );
    }

    #[test]
    fn uint32_decoder_works() {
        assert_decode!(Uint32Decoder, 12345678, [0xce, 0xc2, 0xf1, 0x05]);
    }

    #[test]
    fn uint32_encoder_works() {
        assert_encode!(Uint32Encoder, 12345678, [0xce, 0xc2, 0xf1, 0x05]);
    }

    #[test]
    fn uint64_decoder_works() {
        assert_decode!(Uint64Decoder, 12345678, [0xce, 0xc2, 0xf1, 0x05]);
    }

    #[test]
    fn uint64_encoder_works() {
        assert_encode!(Uint64Encoder, 12345678, [0xce, 0xc2, 0xf1, 0x05]);
    }

    #[test]
    fn sint32_decoder_works() {
        assert_decode!(Sint32Decoder, -1, [0x01]);
        assert_decode!(Sint32Decoder, -12345678, [0x9b, 0x85, 0xe3, 0x0b]);
        assert_decode!(Sint32Decoder, 12345678, [0x9c, 0x85, 0xe3, 0x0b]);
    }

    #[test]
    fn sint32_encoder_works() {
        assert_encode!(Sint32Encoder, -1, [0x01]);
        assert_encode!(Sint32Encoder, -12345678, [0x9b, 0x85, 0xe3, 0x0b]);
        assert_encode!(Sint32Encoder, 12345678, [0x9c, 0x85, 0xe3, 0x0b]);
    }

    #[test]
    fn sint64_decoder_works() {
        assert_decode!(Sint64Decoder, -1, [0x01]);
        assert_decode!(Sint64Decoder, -12345678, [0x9b, 0x85, 0xe3, 0x0b]);
        assert_decode!(Sint64Decoder, 12345678, [0x9c, 0x85, 0xe3, 0x0b]);
    }

    #[test]
    fn sint64_encoder_works() {
        assert_encode!(Sint64Encoder, -1, [0x01]);
        assert_encode!(Sint64Encoder, -12345678, [0x9b, 0x85, 0xe3, 0x0b]);
        assert_encode!(Sint64Encoder, 12345678, [0x9c, 0x85, 0xe3, 0x0b]);
    }

    #[test]
    fn bytes_decoder_works() {
        assert_decode!(BytesDecoder, [0, 1, 2, 3], [4, 0, 1, 2, 3]);
    }

    #[test]
    fn bytes_encoder_works() {
        assert_encode!(BytesEncoder, [0, 1, 2, 3], [4, 0, 1, 2, 3]);
    }

    #[test]
    fn string_decoder_works() {
        assert_decode!(StringDecoder, "foo", [3, 102, 111, 111]);
    }

    #[test]
    fn string_encoder_works() {
        assert_encode!(StringEncoder, "foo", [3, 102, 111, 111]);
    }
}
