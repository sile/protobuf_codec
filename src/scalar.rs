//! Encoders and decoders for [scalar] values.
//!
//! [scalar]: https://developers.google.com/protocol-buffers/docs/proto3#scalar
use bytecodec::bytes::{BytesEncoder as BytesEncoderInner, RemainingBytesDecoder, Utf8Decoder,
                       Utf8Encoder};
use bytecodec::fixnum::{F32leDecoder, F32leEncoder, F64leDecoder, F64leEncoder, I32leDecoder,
                        I32leEncoder, I64leDecoder, I64leEncoder, U32leDecoder, U32leEncoder,
                        U64leDecoder, U64leEncoder};
use bytecodec::{ByteCount, Decode, Encode, Eos, ExactBytesEncode, Result};

use value::{MapKeyDecode, MapKeyEncode, NumericValueDecode, NumericValueEncode,
            OptionalValueDecode, OptionalValueEncode, ValueDecode, ValueEncode};
use wire::{LengthDelimitedDecoder, LengthDelimitedEncoder, VarintDecoder, VarintEncoder, WireType};

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
        impl ValueDecode for $decoder {
            fn wire_type(&self) -> WireType {
                WireType::$wire
            }

            fn merge_values(old: &mut Self::Item, new: Self::Item) {
                *old = new;
            }
        }
        impl OptionalValueDecode for $decoder {
            type Optional = $item;

            fn merge_optional_values(old: &mut Self::Optional, new: Self::Optional) {
                *old = new;
            }
        }
    };
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
        impl ValueEncode for $encoder {
            fn wire_type(&self) -> WireType {
                WireType::$wire
            }
        }
        impl OptionalValueEncode for $encoder {
            type Optional = $item;

            #[cfg_attr(feature = "cargo-clippy", allow(float_cmp))]
            fn start_encoding_if_needed(&mut self, item: Self::Optional) -> Result<()> {
                if item != Self::Item::default() {
                    track!(self.start_encoding(item))?;
                }
                Ok(())
            }
        }
    };
}

macro_rules! impl_varint_decode {
    ($decoder:ty, $item:ty) => {
        impl Decode for $decoder {
            type Item = $item;

            fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
                let (size, item) = track!(self.0.decode(buf, eos))?;
                let item = item.map(Self::value_from_varint);
                Ok((size, item))
            }

            fn requiring_bytes(&self) -> ByteCount {
                self.0.requiring_bytes()
            }
        }
        impl ValueDecode for $decoder {
            fn wire_type(&self) -> WireType {
                WireType::Varint
            }

            fn merge_values(old: &mut Self::Item, new: Self::Item) {
                *old = new;
            }
        }
        impl OptionalValueDecode for $decoder {
            type Optional = $item;

            fn merge_optional_values(old: &mut Self::Optional, new: Self::Optional) {
                *old = new;
            }
        }
    };
}

macro_rules! impl_varint_encode {
    ($encoder:ty, $item:ty) => {
        impl Encode for $encoder {
            type Item = $item;

            fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
                track!(self.0.encode(buf, eos))
            }

            fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
                track!(self.0.start_encoding(Self::value_to_varint(item)))
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
        impl ValueEncode for $encoder {
            fn wire_type(&self) -> WireType {
                WireType::Varint
            }
        }
        impl OptionalValueEncode for $encoder {
            type Optional = $item;

            fn start_encoding_if_needed(&mut self, item: Self::Optional) -> Result<()> {
                if item != Self::Item::default() {
                    track!(self.start_encoding(item))?;
                }
                Ok(())
            }
        }
    };
}

/// Decoder for `double` values.
#[derive(Debug, Default)]
pub struct DoubleDecoder(F64leDecoder);
impl DoubleDecoder {
    /// Makes a new `DoubleDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(DoubleDecoder, f64, Bit64);
impl NumericValueDecode for DoubleDecoder {}

/// Encoder for `double` values.
#[derive(Debug, Default)]
pub struct DoubleEncoder(F64leEncoder);
impl DoubleEncoder {
    /// Makes a new `DoubleEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(DoubleEncoder, f64, Bit64);
impl NumericValueEncode for DoubleEncoder {}

/// Decoder for `float` values.
#[derive(Debug, Default)]
pub struct FloatDecoder(F32leDecoder);
impl FloatDecoder {
    /// Makes a new `FloatDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(FloatDecoder, f32, Bit32);
impl NumericValueDecode for FloatDecoder {}

/// Encoder for `float` values.
#[derive(Debug, Default)]
pub struct FloatEncoder(F32leEncoder);
impl FloatEncoder {
    /// Makes a new `FloatEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(FloatEncoder, f32, Bit32);
impl NumericValueEncode for FloatEncoder {}

/// Decoder for `fixed32` values.
#[derive(Debug, Default)]
pub struct Fixed32Decoder(U32leDecoder);
impl Fixed32Decoder {
    /// Makes a new `Fixed32Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(Fixed32Decoder, u32, Bit32);
impl MapKeyDecode for Fixed32Decoder {}
impl NumericValueDecode for Fixed32Decoder {}

/// Encoder for `fixed32` values.
#[derive(Debug, Default)]
pub struct Fixed32Encoder(U32leEncoder);
impl Fixed32Encoder {
    /// Makes a new `Fixed32Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(Fixed32Encoder, u32, Bit32);
impl MapKeyEncode for Fixed32Encoder {}
impl NumericValueEncode for Fixed32Encoder {}

/// Decoder for `fixed64` values.
#[derive(Debug, Default)]
pub struct Fixed64Decoder(U64leDecoder);
impl Fixed64Decoder {
    /// Makes a new `Fixed64Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(Fixed64Decoder, u64, Bit64);
impl MapKeyDecode for Fixed64Decoder {}
impl NumericValueDecode for Fixed64Decoder {}

/// Encoder for `fixed64` values.
#[derive(Debug, Default)]
pub struct Fixed64Encoder(U64leEncoder);
impl Fixed64Encoder {
    /// Makes a new `Fixed64Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(Fixed64Encoder, u64, Bit64);
impl MapKeyEncode for Fixed64Encoder {}
impl NumericValueEncode for Fixed64Encoder {}

/// Decoder for `sfixed32` values.
#[derive(Debug, Default)]
pub struct Sfixed32Decoder(I32leDecoder);
impl Sfixed32Decoder {
    /// Makes a new `Sfixed32Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(Sfixed32Decoder, i32, Bit32);
impl MapKeyDecode for Sfixed32Decoder {}
impl NumericValueDecode for Sfixed32Decoder {}

/// Encoder for `sfixed32` values.
#[derive(Debug, Default)]
pub struct Sfixed32Encoder(I32leEncoder);
impl Sfixed32Encoder {
    /// Makes a new `Sfixed32Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(Sfixed32Encoder, i32, Bit32);
impl MapKeyEncode for Sfixed32Encoder {}
impl NumericValueEncode for Sfixed32Encoder {}

/// Decoder for `sfixed64` values.
#[derive(Debug, Default)]
pub struct Sfixed64Decoder(I64leDecoder);
impl Sfixed64Decoder {
    /// Makes a new `Sfixed64Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(Sfixed64Decoder, i64, Bit64);
impl MapKeyDecode for Sfixed64Decoder {}
impl NumericValueDecode for Sfixed64Decoder {}

/// Encoder for `sfixed64` values.
#[derive(Debug, Default)]
pub struct Sfixed64Encoder(I64leEncoder);
impl Sfixed64Encoder {
    /// Makes a new `Sfixed64Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_encode!(Sfixed64Encoder, i64, Bit64);
impl MapKeyEncode for Sfixed64Encoder {}
impl NumericValueEncode for Sfixed64Encoder {}

/// Decoder for `bool` values.
#[derive(Debug, Default)]
pub struct BoolDecoder(VarintDecoder);
impl BoolDecoder {
    /// Makes a new `BoolDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_from_varint(n: u64) -> bool {
        n != 0
    }
}
impl_varint_decode!(BoolDecoder, bool);
impl MapKeyDecode for BoolDecoder {}
impl NumericValueDecode for BoolDecoder {}

/// Encoder for `bool` values.
#[derive(Debug, Default)]
pub struct BoolEncoder(VarintEncoder);
impl BoolEncoder {
    /// Makes a new `BoolEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_to_varint(n: bool) -> u64 {
        n as u64
    }
}
impl_varint_encode!(BoolEncoder, bool);
impl MapKeyEncode for BoolEncoder {}
impl NumericValueEncode for BoolEncoder {}

/// Decoder for `int32` values.
#[derive(Debug, Default)]
pub struct Int32Decoder(VarintDecoder);
impl Int32Decoder {
    /// Makes a new `Int32Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_from_varint(n: u64) -> i32 {
        n as i32
    }
}
impl_varint_decode!(Int32Decoder, i32);
impl MapKeyDecode for Int32Decoder {}
impl NumericValueDecode for Int32Decoder {}

/// Encoder for `int32` values.
#[derive(Debug, Default)]
pub struct Int32Encoder(VarintEncoder);
impl Int32Encoder {
    /// Makes a new `Int32Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_to_varint(n: i32) -> u64 {
        n as u64
    }
}
impl_varint_encode!(Int32Encoder, i32);
impl MapKeyEncode for Int32Encoder {}
impl NumericValueEncode for Int32Encoder {}

/// Decoder for `int64` values.
#[derive(Debug, Default)]
pub struct Int64Decoder(VarintDecoder);
impl Int64Decoder {
    /// Makes a new `Int64Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_from_varint(n: u64) -> i64 {
        n as i64
    }
}
impl_varint_decode!(Int64Decoder, i64);
impl MapKeyDecode for Int64Decoder {}
impl NumericValueDecode for Int64Decoder {}

/// Encoder for `int64` values.
#[derive(Debug, Default)]
pub struct Int64Encoder(VarintEncoder);
impl Int64Encoder {
    /// Makes a new `Int64Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_to_varint(n: i64) -> u64 {
        n as u64
    }
}
impl_varint_encode!(Int64Encoder, i64);
impl MapKeyEncode for Int64Encoder {}
impl NumericValueEncode for Int64Encoder {}

/// Decoder for `uint32` values.
#[derive(Debug, Default)]
pub struct Uint32Decoder(VarintDecoder);
impl Uint32Decoder {
    /// Makes a new `Uint32Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_from_varint(n: u64) -> u32 {
        n as u32
    }
}
impl_varint_decode!(Uint32Decoder, u32);
impl MapKeyDecode for Uint32Decoder {}
impl NumericValueDecode for Uint32Decoder {}

/// Encoder for `uint32` values.
#[derive(Debug, Default)]
pub struct Uint32Encoder(VarintEncoder);
impl Uint32Encoder {
    /// Makes a new `Uint32Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_to_varint(n: u32) -> u64 {
        u64::from(n)
    }
}
impl_varint_encode!(Uint32Encoder, u32);
impl MapKeyEncode for Uint32Encoder {}
impl NumericValueEncode for Uint32Encoder {}

/// Decoder for `uint64` values.
#[derive(Debug, Default)]
pub struct Uint64Decoder(VarintDecoder);
impl Uint64Decoder {
    /// Makes a new `Uint64Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_from_varint(n: u64) -> u64 {
        n
    }
}
impl_varint_decode!(Uint64Decoder, u64);
impl MapKeyDecode for Uint64Decoder {}
impl NumericValueDecode for Uint64Decoder {}

/// Encoder for `uint64` values.
#[derive(Debug, Default)]
pub struct Uint64Encoder(VarintEncoder);
impl Uint64Encoder {
    /// Makes a new `Uint64Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_to_varint(n: u64) -> u64 {
        n
    }
}
impl_varint_encode!(Uint64Encoder, u64);
impl MapKeyEncode for Uint64Encoder {}
impl NumericValueEncode for Uint64Encoder {}

/// Decoder for `sint32` values.
#[derive(Debug, Default)]
pub struct Sint32Decoder(VarintDecoder);
impl Sint32Decoder {
    /// Makes a new `Sint32Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_from_varint(n: u64) -> i32 {
        let n = n as i32;
        (n >> 1) ^ ((n << 31) >> 31)
    }
}
impl_varint_decode!(Sint32Decoder, i32);
impl MapKeyDecode for Sint32Decoder {}
impl NumericValueDecode for Sint32Decoder {}

/// Encoder for `sint32` values.
#[derive(Debug, Default)]
pub struct Sint32Encoder(VarintEncoder);
impl Sint32Encoder {
    /// Makes a new `Sint32Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_to_varint(n: i32) -> u64 {
        ((n << 1) ^ (n >> 31)) as u64
    }
}
impl_varint_encode!(Sint32Encoder, i32);
impl MapKeyEncode for Sint32Encoder {}
impl NumericValueEncode for Sint32Encoder {}

/// Decoder for `sint64` values.
#[derive(Debug, Default)]
pub struct Sint64Decoder(VarintDecoder);
impl Sint64Decoder {
    /// Makes a new `Sint64Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_from_varint(n: u64) -> i64 {
        let n = n as i64;
        (n >> 1) ^ ((n << 63) >> 63)
    }
}
impl_varint_decode!(Sint64Decoder, i64);
impl MapKeyDecode for Sint64Decoder {}
impl NumericValueDecode for Sint64Decoder {}

/// Encoder for `sint64` values.
#[derive(Debug, Default)]
pub struct Sint64Encoder(VarintEncoder);
impl Sint64Encoder {
    /// Makes a new `Sint64Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn value_to_varint(n: i64) -> u64 {
        ((n << 1) ^ (n >> 63)) as u64
    }
}
impl_varint_encode!(Sint64Encoder, i64);
impl MapKeyEncode for Sint64Encoder {}
impl NumericValueEncode for Sint64Encoder {}

/// Decoder for `bytes` values.
#[derive(Debug, Default)]
pub struct BytesDecoder(LengthDelimitedDecoder<RemainingBytesDecoder>);
impl BytesDecoder {
    /// Makes a new `BytesDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(BytesDecoder, Vec<u8>, LengthDelimited);

/// Decoder for custom `bytes` values.
///
/// This is equivalent to `BytesDecoder` in the protobol buffers layer,
/// but it decodes the payload bytes by using `D` and
/// returns the decoded items to the application layer instead of raw bytes.
#[derive(Debug, Default)]
pub struct CustomBytesDecoder<D>(LengthDelimitedDecoder<D>);
impl<D: Decode> CustomBytesDecoder<D> {
    /// Makes a new `CustomBytesDecoder` instance.
    pub fn new(inner: D) -> Self {
        CustomBytesDecoder(LengthDelimitedDecoder::new(inner))
    }

    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        self.0.inner_ref()
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        self.0.inner_mut()
    }

    /// Takes ownership of the instance and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.0.into_inner()
    }
}
impl<D: Decode> Decode for CustomBytesDecoder<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track!(self.0.decode(buf, eos))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<D: Decode> ValueDecode for CustomBytesDecoder<D> {
    fn wire_type(&self) -> WireType {
        WireType::LengthDelimited
    }

    fn merge_values(old: &mut Self::Item, new: Self::Item) {
        *old = new;
    }
}

/// Encoder for `bytes` values.
#[derive(Debug)]
pub struct BytesEncoder<B = Vec<u8>>(LengthDelimitedEncoder<BytesEncoderInner<B>>);
impl<B> BytesEncoder<B> {
    /// Makes a new `BytesEncoder` instance.
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
impl<B: AsRef<[u8]>> ValueEncode for BytesEncoder<B> {
    fn wire_type(&self) -> WireType {
        WireType::LengthDelimited
    }
}
impl<B: AsRef<[u8]>> OptionalValueEncode for BytesEncoder<B> {
    type Optional = B;

    fn start_encoding_if_needed(&mut self, item: Self::Optional) -> Result<()> {
        if !item.as_ref().is_empty() {
            track!(self.start_encoding(item))?;
        }
        Ok(())
    }
}

/// Encoder for custom `bytes` values.
///
/// This is equivalent to `BytesEncoder` in the protobol buffers layer,
/// but it uses the encoder `E` for producing bytes instead of passing raw bytes.
#[derive(Debug, Default)]
pub struct CustomBytesEncoder<E>(LengthDelimitedEncoder<E>);
impl<E: ExactBytesEncode> CustomBytesEncoder<E> {
    /// Makes a new `CustomBytesEncoder` instance.
    pub fn new(inner: E) -> Self {
        CustomBytesEncoder(LengthDelimitedEncoder::new(inner))
    }

    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        self.0.inner_ref()
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        self.0.inner_mut()
    }

    /// Takes ownership of the instance and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.0.into_inner()
    }
}
impl<E: ExactBytesEncode> Encode for CustomBytesEncoder<E> {
    type Item = E::Item;

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
impl<E: ExactBytesEncode> ExactBytesEncode for CustomBytesEncoder<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}
impl<E: ExactBytesEncode> ValueEncode for CustomBytesEncoder<E> {
    fn wire_type(&self) -> WireType {
        WireType::LengthDelimited
    }
}

/// Decoder for `string` values.
#[derive(Debug, Default)]
pub struct StringDecoder(LengthDelimitedDecoder<Utf8Decoder>);
impl StringDecoder {
    /// Makes a new `StringDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl_newtype_decode!(StringDecoder, String, LengthDelimited);
impl MapKeyDecode for StringDecoder {}

/// Encoder for `string` values.
#[derive(Debug)]
pub struct StringEncoder<S = String>(LengthDelimitedEncoder<Utf8Encoder<S>>);
impl<S> StringEncoder<S> {
    /// Makes a new `StringEncoder` instance.
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
impl<S: AsRef<str>> ValueEncode for StringEncoder<S> {
    fn wire_type(&self) -> WireType {
        WireType::LengthDelimited
    }
}
impl<S: AsRef<str>> OptionalValueEncode for StringEncoder<S> {
    type Optional = S;

    fn start_encoding_if_needed(&mut self, item: Self::Optional) -> Result<()> {
        if !item.as_ref().is_empty() {
            track!(self.start_encoding(item))?;
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
        };
    }

    macro_rules! assert_encode {
        ($encoder:ident, $value:expr, $bytes:expr) => {
            let mut buf = Vec::new();
            let mut encoder = track_try_unwrap!($encoder::with_item($value));
            track_try_unwrap!(encoder.encode_all(&mut buf));
            assert_eq!(buf, $bytes);
        };
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
