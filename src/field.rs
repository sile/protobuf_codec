//! Encoders, decoders and related components for message fields.
use bytecodec::bytes::CopyableBytesDecoder;
use bytecodec::combinator::SkipRemaining;
use bytecodec::value::NullDecoder;
use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, ExactBytesEncode, Result};
use std::fmt;

pub use fields::Fields;
pub use oneof::Oneof;
pub use repeated_field::{MapFieldDecoder, MapFieldEncoder, PackedRepeatedFieldDecoder,
                         PackedRepeatedFieldEncoder, RepeatedFieldDecoder, RepeatedFieldEncoder,
                         RepeatedNumericFieldDecoder};

pub mod num {
    //! Field number.

    pub use field_num::*;
}
pub mod branch {
    //! Values for `Oneof` fields.

    pub use oneof::{Branch1, Branch2, Branch3, Branch4, Branch5, Branch6, Branch7, Branch8};
}
pub mod value {
    //! Traits for representing encoders and decoders of field values.

    pub use value::*;
}

use field_num::FieldNum;
use message::{EmbeddedMessageDecoder, EmbeddedMessageEncoder};
use value::{OptionalValueDecode, OptionalValueEncode, ValueDecode, ValueEncode};
use wire::{LengthDelimitedDecoder, Tag, TagEncoder, VarintDecoder, WireType};

/// Decoder for fields that have embedded messages as the value.
pub type MessageFieldDecoder<F, D> = FieldDecoder<F, EmbeddedMessageDecoder<D>>;

/// Decoder for repeated fields that have embedded messages as the value.
pub type RepeatedMessageFieldDecoder<F, V, E> =
    RepeatedFieldDecoder<F, V, EmbeddedMessageDecoder<E>>;

/// Encoder for fields that have embedded messages as the value.
pub type MessageFieldEncoder<F, E> = FieldEncoder<F, EmbeddedMessageEncoder<E>>;

/// Encoder for repeated fields that have embedded messages as the value.
pub type RepeatedMessageFieldEncoder<F, V, E> =
    RepeatedFieldEncoder<F, V, EmbeddedMessageEncoder<E>>;

/// This trait allows for decoding message fields.
pub trait FieldDecode {
    /// The type of the decoded items (i.e., field values).
    type Item;

    /// Tries to start decoding a field.
    ///
    /// If `tag` is not a target of the decoder, `Ok(false)` will be returned.
    fn start_decoding(&mut self, tag: Tag) -> Result<bool>;

    /// Decodes the given bytes.
    ///
    /// This is equivalent to `Decode::decode` method except
    /// this does not return the decoded items as the result value of the method.
    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize>;

    /// Returns `true` if the decoder is in the middle of decoding an item, otherwise `false`.
    fn is_decoding(&self) -> bool;

    /// Takes the item decoded by the decoder.
    ///
    /// Some implementation may returns the default value if the field is missing.
    fn finish_decoding(&mut self) -> Result<Self::Item>;

    /// Returns the lower bound of the number of bytes needed to decode the next item.
    fn requiring_bytes(&self) -> ByteCount;

    /// Merges duplicate field values.
    fn merge_fields(old: &mut Self::Item, new: Self::Item);
}

/// This trait allows for decoding `Oneof` fields.
pub trait OneofFieldDecode: FieldDecode {}

/// This trait allows for encoding message fields.
pub trait FieldEncode: Encode {}

/// This trait allows for encoding `Oneof` fields.
pub trait OneofFieldEncode: FieldEncode {}

/// Decoder for required fields.
#[derive(Debug)]
pub struct FieldDecoder<F, D: ValueDecode> {
    num: F,
    value: D,
    decoded: Option<D::Item>,
    is_decoding: bool,
}
impl<F, D: ValueDecode> FieldDecoder<F, D> {
    /// Makes a new `FieldDecoder` instance.
    pub fn new(field_num: F, value_decoder: D) -> Self {
        FieldDecoder {
            num: field_num,
            value: value_decoder,
            decoded: None,
            is_decoding: false,
        }
    }
}
impl<F, D> FieldDecode for FieldDecoder<F, D>
where
    F: Copy + Into<FieldNum>,
    D: ValueDecode,
{
    type Item = D::Item;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.num.into() == tag.field_num {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track_assert_eq!(self.value.wire_type(), tag.wire_type, ErrorKind::InvalidInput; tag);
            self.is_decoding = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let (size, item) = track!(self.value.decode(buf, eos); self.num.into())?;
        if let Some(new) = item {
            self.is_decoding = false;
            if self.decoded.is_none() {
                self.decoded = Some(new);
            } else {
                let old = self.decoded.as_mut().expect("Never fails");
                D::merge_values(old, new);
            }
        }
        Ok(size)
    }

    fn is_decoding(&self) -> bool {
        self.is_decoding
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding, ErrorKind::Other);
        let v = track_assert_some!(
            self.decoded.take(),
            ErrorKind::InvalidInput,
            "Missing required field: {:?}",
            self.num.into()
        );
        Ok(v)
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.value.requiring_bytes()
    }

    fn merge_fields(old: &mut Self::Item, new: Self::Item) {
        D::merge_values(old, new)
    }
}
impl<F, D> OneofFieldDecode for FieldDecoder<F, D>
where
    F: Copy + Into<FieldNum>,
    D: ValueDecode,
{
}
impl<F, D> Default for FieldDecoder<F, D>
where
    F: Default,
    D: Default + ValueDecode,
{
    fn default() -> Self {
        FieldDecoder {
            num: F::default(),
            value: D::default(),
            decoded: None,
            is_decoding: false,
        }
    }
}

/// Decoder for optional fields.
#[derive(Default)]
pub struct OptionalFieldDecoder<F, D: OptionalValueDecode>(FieldDecoder<F, D>);
impl<F, D: OptionalValueDecode> OptionalFieldDecoder<F, D> {
    /// Makes a new `OptionalFieldDecoder` instance.
    pub fn new(field_num: F, value_decoder: D) -> Self {
        OptionalFieldDecoder(FieldDecoder::new(field_num, value_decoder))
    }
}
impl<F, D> FieldDecode for OptionalFieldDecoder<F, D>
where
    F: Copy + Into<FieldNum>,
    D: OptionalValueDecode,
{
    type Item = D::Optional;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        track!(self.0.start_decoding(tag))
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.0.field_decode(buf, eos))
    }

    fn is_decoding(&self) -> bool {
        self.0.is_decoding()
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding(), ErrorKind::Other);
        if let Some(value) = self.0.decoded.take() {
            Ok(value.into())
        } else {
            Ok(Default::default())
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn merge_fields(old: &mut Self::Item, new: Self::Item) {
        D::merge_optional_values(old, new)
    }
}
impl<F, D> fmt::Debug for OptionalFieldDecoder<F, D>
where
    F: fmt::Debug,
    D: fmt::Debug + OptionalValueDecode,
    D::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OptionalFieldDecoder({:?})", self.0)
    }
}

/// Decoder for unknown fields.
///
/// This accepts any tags but the decoded values will be discarded.
#[derive(Debug)]
pub struct UnknownFieldDecoder(UnknownFieldDecoderInner);
impl UnknownFieldDecoder {
    /// Makes a new `UnknownFieldDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl FieldDecode for UnknownFieldDecoder {
    type Item = ();

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        self.0 = match tag.wire_type {
            WireType::Varint => UnknownFieldDecoderInner::Varint(Default::default()),
            WireType::Bit32 => UnknownFieldDecoderInner::Bit32(Default::default()),
            WireType::Bit64 => UnknownFieldDecoderInner::Bit64(Default::default()),
            WireType::LengthDelimited => {
                UnknownFieldDecoderInner::LengthDelimited(Default::default())
            }
        };
        Ok(true)
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let (size, decoded) = match self.0 {
            UnknownFieldDecoderInner::None => (0, false),
            UnknownFieldDecoderInner::Varint(ref mut d) => {
                track!(d.decode(buf, eos)).map(|(n, i)| (n, i.is_some()))?
            }
            UnknownFieldDecoderInner::Bit32(ref mut d) => {
                track!(d.decode(buf, eos)).map(|(n, i)| (n, i.is_some()))?
            }
            UnknownFieldDecoderInner::Bit64(ref mut d) => {
                track!(d.decode(buf, eos)).map(|(n, i)| (n, i.is_some()))?
            }
            UnknownFieldDecoderInner::LengthDelimited(ref mut d) => {
                track!(d.decode(buf, eos)).map(|(n, i)| (n, i.is_some()))?
            }
        };
        if decoded {
            self.0 = UnknownFieldDecoderInner::None;
        }
        Ok(size)
    }

    fn is_decoding(&self) -> bool {
        if let UnknownFieldDecoderInner::None = self.0 {
            false
        } else {
            true
        }
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding(), ErrorKind::Other);
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        match self.0 {
            UnknownFieldDecoderInner::None => ByteCount::Unknown,
            UnknownFieldDecoderInner::Varint(ref d) => d.requiring_bytes(),
            UnknownFieldDecoderInner::Bit32(ref d) => d.requiring_bytes(),
            UnknownFieldDecoderInner::Bit64(ref d) => d.requiring_bytes(),
            UnknownFieldDecoderInner::LengthDelimited(ref d) => d.requiring_bytes(),
        }
    }

    fn merge_fields(_: &mut Self::Item, _: Self::Item) {}
}
impl Default for UnknownFieldDecoder {
    fn default() -> Self {
        UnknownFieldDecoder(UnknownFieldDecoderInner::None)
    }
}

#[derive(Debug)]
enum UnknownFieldDecoderInner {
    None,
    Varint(VarintDecoder),
    Bit32(CopyableBytesDecoder<[u8; 4]>),
    Bit64(CopyableBytesDecoder<[u8; 8]>),
    LengthDelimited(LengthDelimitedDecoder<SkipRemaining<NullDecoder>>),
}

/// Encoder for optional fields.
#[derive(Debug, Default)]
pub struct OptionalFieldEncoder<F, E>(FieldEncoder<F, E>);
impl<F, E: OptionalValueEncode> OptionalFieldEncoder<F, E> {
    /// Makes a new `OptionalFieldEncoder` instance.
    pub fn new(field_num: F, value_encoder: E) -> Self {
        OptionalFieldEncoder(FieldEncoder::new(field_num, value_encoder))
    }
}
impl<F, E> Encode for OptionalFieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: OptionalValueEncode,
{
    type Item = E::Optional;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.0.value.start_encoding_if_needed(item))?;
        if !self.0.value.is_idle() {
            let tag = Tag::from((self.0.num.into(), self.0.value.wire_type()));
            track!(self.0.tag.start_encoding(tag))?;
        }
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<F, E> ExactBytesEncode for OptionalFieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: ExactBytesEncode + OptionalValueEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}
impl<F, E> FieldEncode for OptionalFieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: OptionalValueEncode,
{
}

/// Encoder for required fields.
#[derive(Debug, Default)]
pub struct FieldEncoder<F, E> {
    num: F,
    tag: TagEncoder,
    value: E,
}
impl<F, E> FieldEncoder<F, E>
where
    E: ValueEncode,
{
    /// Makes a new `FieldEncoder` instance.
    pub fn new(field_num: F, value_encoder: E) -> Self {
        FieldEncoder {
            num: field_num,
            tag: TagEncoder::new(),
            value: value_encoder,
        }
    }
}
impl<F, E> Encode for FieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: ValueEncode,
{
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.tag, offset, buf, eos);
        bytecodec_try_encode!(self.value, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let tag = Tag::from((self.num.into(), self.value.wire_type()));
        track!(self.tag.start_encoding(tag))?;
        track!(self.value.start_encoding(item))?;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.value.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.tag
            .requiring_bytes()
            .add_for_encoding(self.value.requiring_bytes())
    }
}
impl<F, E> ExactBytesEncode for FieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: ExactBytesEncode + ValueEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.tag.exact_requiring_bytes() + self.value.exact_requiring_bytes()
    }
}
impl<F, E> FieldEncode for FieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: ValueEncode,
{
}

#[cfg(test)]
mod test {
    use bytecodec::EncodeExt;
    use bytecodec::io::IoEncodeExt;

    use super::*;
    use scalar::Fixed32Encoder;

    macro_rules! assert_encode {
        ($encoder:ty, $value:expr, $bytes:expr) => {
            let mut buf = Vec::new();
            let mut encoder: $encoder = track_try_unwrap!(EncodeExt::with_item($value));
            track_try_unwrap!(encoder.encode_all(&mut buf));
            assert_eq!(buf, $bytes);
        };
    }

    #[test]
    fn field_encoder_works() {
        assert_encode!(FieldEncoder<num::F1, Fixed32Encoder>, 123, [0x0d, 0x7b, 0x00, 0x00, 0x00]);
    }
}
