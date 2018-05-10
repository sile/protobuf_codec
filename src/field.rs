use std::fmt;
use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, ExactBytesEncode, Result};
use bytecodec::bytes::CopyableBytesDecoder;
use bytecodec::combinator::SkipRemaining;
use bytecodec::value::NullDecoder;

pub use fields::Fields;
pub use oneof::{OneOf, OneOf2, OneOf3, OneOf4, OneOf5, OneOf6, OneOf7, OneOf8};
pub use repeated_field::{MapFieldDecoder, MapFieldEncoder, PackedRepeatedFieldDecoder,
                         PackedRepeatedFieldEncoder, RepeatedFieldDecoder, RepeatedFieldEncoder,
                         RepeatedNumericFieldDecoder};

use tag::Tag;
use message::{EmbeddedMessageDecoder, EmbeddedMessageEncoder};
use value::{OptionalValueDecode, OptionalValueEncode, ValueDecode, ValueEncode};
use wire::{LengthDelimitedDecoder, TagAndTypeEncoder, VarintDecoder, WireType};

pub type MessageFieldDecoder<T, D> = FieldDecoder<T, EmbeddedMessageDecoder<D>>;
pub type RepeatedMessageFieldDecoder<T, V, E> =
    RepeatedFieldDecoder<T, V, EmbeddedMessageDecoder<E>>;

pub type MessageFieldEncoder<T, E> = FieldEncoder<T, EmbeddedMessageEncoder<E>>;
pub type RepeatedMessageFieldEncoder<T, V, E> =
    RepeatedFieldEncoder<T, V, EmbeddedMessageEncoder<E>>;

pub trait FieldDecode {
    type Item;

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool>;
    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize>;
    fn is_decoding(&self) -> bool;
    fn finish_decoding(&mut self) -> Result<Self::Item>;
    fn requiring_bytes(&self) -> ByteCount;
    fn merge_fields(old: &mut Self::Item, new: Self::Item);
}

pub trait OneOfFieldDecode: FieldDecode {}

pub trait FieldEncode: Encode {}

pub trait OneOfFieldEncode: FieldEncode {}

#[derive(Debug)]
pub struct FieldDecoder<T, D: ValueDecode> {
    tag: T,
    decoder: D,
    value: Option<D::Item>,
    is_decoding: bool,
}
impl<T, D> FieldDecode for FieldDecoder<T, D>
where
    T: Copy + Into<Tag>,
    D: ValueDecode,
{
    type Item = D::Item;

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
        if self.tag.into() == tag {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track_assert_eq!(self.decoder.wire_type(), wire_type, ErrorKind::InvalidInput; tag);
            self.is_decoding = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let (size, item) = track!(self.decoder.decode(buf, eos); self.tag.into())?;
        if let Some(new) = item {
            self.is_decoding = false;
            if self.value.is_none() {
                self.value = Some(new);
            } else {
                self.value.as_mut().map(|old| D::merge_values(old, new));
            }
        }
        Ok(size)
    }

    fn is_decoding(&self) -> bool {
        self.is_decoding
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding, ErrorKind::Other);
        let value = track_assert_some!(
            self.value.take(),
            ErrorKind::InvalidInput,
            "Missing required field: {:?}",
            self.tag.into()
        );
        Ok(value)
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.decoder.requiring_bytes()
    }

    fn merge_fields(old: &mut Self::Item, new: Self::Item) {
        D::merge_values(old, new)
    }
}
impl<T, D> OneOfFieldDecode for FieldDecoder<T, D>
where
    T: Copy + Into<Tag>,
    D: ValueDecode,
{
}
impl<T, D> Default for FieldDecoder<T, D>
where
    T: Default,
    D: Default + ValueDecode,
{
    fn default() -> Self {
        FieldDecoder {
            tag: T::default(),
            decoder: D::default(),
            value: None,
            is_decoding: false,
        }
    }
}

#[derive(Default)]
pub struct OptionalFieldDecoder<T, D: OptionalValueDecode>(FieldDecoder<T, D>);
impl<T, D> FieldDecode for OptionalFieldDecoder<T, D>
where
    T: Copy + Into<Tag>,
    D: OptionalValueDecode,
{
    type Item = D::Optional;

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
        track!(self.0.start_decoding(tag, wire_type))
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.0.field_decode(buf, eos))
    }

    fn is_decoding(&self) -> bool {
        self.0.is_decoding()
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding(), ErrorKind::Other);
        if let Some(value) = self.0.value.take() {
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
impl<T, D> fmt::Debug for OptionalFieldDecoder<T, D>
where
    T: fmt::Debug,
    D: fmt::Debug + OptionalValueDecode,
    D::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OptionalFieldDecoder({:?})", self.0)
    }
}

#[derive(Debug)]
pub struct UnknownFieldDecoder(UnknownFieldDecoderInner);
impl FieldDecode for UnknownFieldDecoder {
    type Item = ();

    fn start_decoding(&mut self, _tag: Tag, wire_type: WireType) -> Result<bool> {
        self.0 = match wire_type {
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

#[derive(Debug, Default)]
pub struct OptionalFieldEncoder<T, E>(FieldEncoder<T, E>);
impl<T, E> Encode for OptionalFieldEncoder<T, E>
where
    T: Copy + Into<Tag>,
    E: OptionalValueEncode,
{
    type Item = E::Optional;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.0.value.start_encoding_if_needed(item))?;
        if !self.0.value.is_idle() {
            let tag_and_type = (self.0.tag.into(), self.0.value.wire_type());
            track!(self.0.tag_and_type.start_encoding(tag_and_type))?;
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
impl<T, E> ExactBytesEncode for OptionalFieldEncoder<T, E>
where
    T: Copy + Into<Tag>,
    E: ExactBytesEncode + OptionalValueEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}
impl<T, E> FieldEncode for OptionalFieldEncoder<T, E>
where
    T: Copy + Into<Tag>,
    E: OptionalValueEncode,
{
}

#[derive(Debug, Default)]
pub struct FieldEncoder<T, E> {
    tag: T,
    tag_and_type: TagAndTypeEncoder,
    value: E,
}
impl<T, E> FieldEncoder<T, E>
where
    T: Copy + Into<Tag>,
    E: ValueEncode,
{
    pub fn new(tag: T, value_encoder: E) -> Self {
        FieldEncoder {
            tag,
            tag_and_type: TagAndTypeEncoder::new(),
            value: value_encoder,
        }
    }
}
impl<T, E> Encode for FieldEncoder<T, E>
where
    T: Copy + Into<Tag>,
    E: ValueEncode,
{
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.tag_and_type, offset, buf, eos);
        bytecodec_try_encode!(self.value, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let tag_and_type = (self.tag.into(), self.value.wire_type());
        track!(self.tag_and_type.start_encoding(tag_and_type))?;
        track!(self.value.start_encoding(item))?;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.value.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.tag_and_type
            .requiring_bytes()
            .add_for_encoding(self.value.requiring_bytes())
    }
}
impl<T, E> ExactBytesEncode for FieldEncoder<T, E>
where
    T: Copy + Into<Tag>,
    E: ExactBytesEncode + ValueEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.tag_and_type.exact_requiring_bytes() + self.value.exact_requiring_bytes()
    }
}
impl<T, E> FieldEncode for FieldEncoder<T, E>
where
    T: Copy + Into<Tag>,
    E: ValueEncode,
{
}

#[cfg(test)]
mod test {
    use bytecodec::EncodeExt;
    use bytecodec::io::IoEncodeExt;

    use scalar::Fixed32Encoder;
    use tag::Tag1;
    use super::*;

    macro_rules! assert_encode {
        ($encoder:ty, $value:expr, $bytes:expr) => {
            let mut buf = Vec::new();
            let mut encoder: $encoder = track_try_unwrap!(EncodeExt::with_item($value));
            track_try_unwrap!(encoder.encode_all(&mut buf));
            assert_eq!(buf, $bytes);
        }
    }

    #[test]
    fn field_encoder_works() {
        assert_encode!(FieldEncoder<Tag1, Fixed32Encoder>, 123, [0x0d, 0x7b, 0x00, 0x00, 0x00]);
    }
}
