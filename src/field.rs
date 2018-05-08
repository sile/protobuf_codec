use std::iter;
use std::mem;
use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, ExactBytesEncode, Result};
use bytecodec::bytes::CopyableBytesDecoder;
use bytecodec::combinator::SkipRemaining;
use bytecodec::value::NullDecoder;

pub use fields::FieldsDecoder;

use message::{EmbeddedMessageDecoder, EmbeddedMessageEncoder};
use scalar::NumericEncode;
use tag::Tag;
use wire::{LengthDelimitedDecoder, TagAndTypeEncoder, VarintDecoder, WireDecode, WireEncode,
           WireType};

pub trait FieldDecode {
    type Item;

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool>;
    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize>;
    fn is_decoding(&self) -> bool;
    fn finish_decoding(&mut self) -> Result<Self::Item>;
    fn requiring_bytes(&self) -> ByteCount;
    fn merge(&self, old: Self::Item, new: Self::Item) -> Self::Item;
}

pub trait FieldEncode: Encode {}

pub type MessageFieldDecoder<T, M> = FieldDecoder<T, EmbeddedMessageDecoder<M>>;

#[derive(Debug, Default)]
pub struct FieldDecoder<T, D: WireDecode> {
    tag: T,
    decoder: D,
    value: D::Value,
    is_decoding: bool,
}
impl<T, D: WireDecode> FieldDecoder<T, D> {
    pub fn new(tag: T, value_decoder: D) -> Self {
        FieldDecoder {
            tag,
            decoder: value_decoder,
            value: D::Value::default(),
            is_decoding: false,
        }
    }
}
impl<T, D> FieldDecode for FieldDecoder<T, D>
where
    T: Copy + Into<Tag>,
    D: WireDecode,
{
    type Item = D::Value;

    fn start_decoding(&mut self, tag: Tag, _: WireType) -> Result<bool> {
        if self.tag.into() != tag {
            Ok(false)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track_assert!(self.is_decoding, ErrorKind::Other);

        let (size, item) = track!(self.decoder.decode(buf, eos); self.tag.into())?;
        if let Some(new) = item {
            self.is_decoding = false;
            self.value = self.decoder.merge(
                mem::replace(&mut self.value, D::Value::default()),
                new.into(),
            );
        }
        Ok(size)
    }

    fn is_decoding(&self) -> bool {
        self.is_decoding
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding, ErrorKind::InvalidInput);
        let value = mem::replace(&mut self.value, D::Value::default());
        Ok(value)
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_decoding {
            self.decoder.requiring_bytes()
        } else {
            ByteCount::Finite(0)
        }
    }

    fn merge(&self, old: Self::Item, new: Self::Item) -> Self::Item {
        self.decoder.merge(old, new)
    }
}

#[derive(Debug, Default)]
pub struct RepeatedFieldDecoder<T, V, D> {
    tag: T,
    decoder: D,
    values: V,
    is_decoding: bool,
}
impl<T, V, D> FieldDecode for RepeatedFieldDecoder<T, V, D>
where
    T: Copy + Into<Tag>,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
    D: WireDecode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag, _: WireType) -> Result<bool> {
        if self.tag.into() != tag {
            Ok(false)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track_assert!(self.is_decoding, ErrorKind::Other);

        let (size, item) = track!(self.decoder.decode(buf, eos); self.tag.into())?;
        if let Some(value) = item {
            self.values.extend(iter::once(value));
            self.is_decoding = false;
        }
        Ok(size)
    }

    fn is_decoding(&self) -> bool {
        self.is_decoding
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding, ErrorKind::InvalidInput);
        let values = mem::replace(&mut self.values, V::default());
        Ok(values)
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_decoding {
            self.decoder.requiring_bytes()
        } else {
            ByteCount::Finite(0)
        }
    }

    fn merge(&self, mut old: Self::Item, new: Self::Item) -> Self::Item {
        old.extend(new.into_iter());
        old
    }
}

#[derive(Debug)]
pub struct UnknownFieldDecoder(UnknownFieldDecoderInner);
impl FieldDecode for UnknownFieldDecoder {
    type Item = ();

    fn start_decoding(&mut self, _tag: Tag, wire_type: WireType) -> Result<bool> {
        track_assert!(!self.is_decoding(), ErrorKind::Other);
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

    fn merge(&self, (): Self::Item, (): Self::Item) -> Self::Item {
        ()
    }
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

pub type MessageFieldEncoder<T, M> = FieldEncoder<T, EmbeddedMessageEncoder<M>>;

#[derive(Debug, Default)]
pub struct FieldEncoder<T, E> {
    tag: T,
    tag_and_type: TagAndTypeEncoder,
    value: E,
}
impl<T, E> FieldEncoder<T, E>
where
    T: Copy + Into<Tag>,
    E: WireEncode,
{
    pub fn new(tag: T, value_encoder: E) -> Self {
        FieldEncoder {
            tag,
            tag_and_type: TagAndTypeEncoder::new(),
            value: value_encoder,
        }
    }

    fn force_start_encoding(&mut self, item: E::Item) -> Result<()> {
        let tag_and_type = (self.tag.into(), self.value.wire_type());
        track!(self.tag_and_type.start_encoding(tag_and_type))?;
        track!(self.value.start_encoding(item))?;
        Ok(())
    }
}
impl<T, E> Encode for FieldEncoder<T, E>
where
    T: Copy + Into<Tag>,
    E: WireEncode,
{
    type Item = E::Value;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.tag_and_type, offset, buf, eos);
        bytecodec_try_encode!(self.value, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.value.start_encoding_value(item))?;
        if !self.value.is_idle() {
            let tag_and_type = (self.tag.into(), self.value.wire_type());
            track!(self.tag_and_type.start_encoding(tag_and_type))?;
        }
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
    E: ExactBytesEncode + WireEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.tag_and_type.exact_requiring_bytes() + self.value.exact_requiring_bytes()
    }
}
impl<T, E> FieldEncode for FieldEncoder<T, E>
where
    T: Copy + Into<Tag>,
    E: WireEncode,
{
}

#[derive(Debug)]
pub struct RepeatedFieldEncoder<T, F: IntoIterator, E> {
    inner: FieldEncoder<T, E>,
    values: Option<F::IntoIter>,
}
impl<T: Default, F: IntoIterator, E: Default> Default for RepeatedFieldEncoder<T, F, E> {
    fn default() -> Self {
        RepeatedFieldEncoder {
            inner: FieldEncoder::default(),
            values: None,
        }
    }
}
impl<T, F, E> Encode for RepeatedFieldEncoder<T, F, E>
where
    T: Copy + Into<Tag>,
    F: IntoIterator,
    E: WireEncode<Item = F::Item>,
{
    type Item = F;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        while offset < buf.len() {
            if self.inner.is_idle() {
                if let Some(item) = self.values.as_mut().and_then(|x| x.next()) {
                    track!(self.inner.force_start_encoding(item))?;
                } else {
                    self.values = None;
                    break;
                }
            }
            bytecodec_try_encode!(self.inner, offset, buf, eos);
        }
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        self.values = Some(item.into_iter());
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.values.is_none()
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_idle() {
            ByteCount::Finite(0)
        } else {
            ByteCount::Unknown
        }
    }
}
impl<T, F, E> FieldEncode for RepeatedFieldEncoder<T, F, E>
where
    T: Copy + Into<Tag>,
    F: IntoIterator,
    E: WireEncode<Item = F::Item>,
{
}

#[derive(Debug)]
pub struct PackedRepeatedFieldEncoder<T, F: IntoIterator, E> {
    tag: T,
    values: Option<F::IntoIter>,
    tag_and_type_encoder: TagAndTypeEncoder,
    value_encoder: E,
}
impl<T: Default, F: IntoIterator, E: Default> Default for PackedRepeatedFieldEncoder<T, F, E> {
    fn default() -> Self {
        PackedRepeatedFieldEncoder {
            tag: T::default(),
            values: None,
            tag_and_type_encoder: TagAndTypeEncoder::new(),
            value_encoder: E::default(),
        }
    }
}
impl<T, F, E> Encode for PackedRepeatedFieldEncoder<T, F, E>
where
    T: Copy + Into<Tag>,
    F: IntoIterator,
    E: NumericEncode<Item = F::Item>,
{
    type Item = F;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.tag_and_type_encoder, offset, buf, eos);

        while offset < buf.len() {
            if self.value_encoder.is_idle() {
                if let Some(item) = self.values.as_mut().and_then(|x| x.next()) {
                    track!(self.value_encoder.start_encoding(item))?;
                } else {
                    self.values = None;
                    break;
                }
            }
            bytecodec_try_encode!(self.value_encoder, offset, buf, eos);
        }
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        let tag_and_type = (self.tag.into(), self.value_encoder.wire_type());
        track!(self.tag_and_type_encoder.start_encoding(tag_and_type))?;
        self.values = Some(item.into_iter());
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.values.is_none()
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_idle() {
            ByteCount::Finite(0)
        } else {
            ByteCount::Unknown
        }
    }
}
impl<T, F, E> FieldEncode for PackedRepeatedFieldEncoder<T, F, E>
where
    T: Copy + Into<Tag>,
    F: IntoIterator,
    E: NumericEncode<Item = F::Item>,
{
}

// // Only repeated fields of primitive numeric types
// // (types which use the varint, 32-bit, or 64-bit wire types) can be declared "packed".
// //
// // Protocol buffer parsers must be able to parse repeated fields that were compiled as packed
// // as if they were not packed, and vice versa.
// // This permits adding [packed=true] to existing fields in a forward- and backward-compatible way.
// #[derive(Debug, Default)]
// pub struct PackedRepeatedFieldDecoder<T, V, D>
// where
//     V: Default + Extend<D::Item>,
//     D: Decode,
// {
//     tag: T,
//     value: Buffered<LengthDelimitedDecoder<Collect<D, V>>>,
//     is_decoding: bool,
// }
// impl<T, V, D> FieldDecode for PackedRepeatedFieldDecoder<T, V, D>
// where
//     T: Copy + Into<Tag>,
//     V: Default + Extend<D::Item>,
//     D: Decode,
// {
//     type Item = V;

//     fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
//         if self.tag.into() != tag {
//             Ok(false)
//         } else {
//             track_assert!(!self.is_decoding, ErrorKind::Other);
//             // TODO:
//             // > For numeric types and strings, if the same field appears multiple times,
//             // > the parser accepts the last value it sees. For embedded message fields,
//             // > the parser merges multiple instances of the same field,
//             // > as if with the Message::MergeFrom method
//             track_assert!(
//                 self.value.has_item(),
//                 ErrorKind::InvalidInput,
//                 "This field can be appeared at most once: tag={}",
//                 self.tag.into().0
//             );
//             self.is_decoding = true;
//             Ok(true)
//         }
//     }

//     fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
//         track_assert!(self.is_decoding, ErrorKind::Other);

//         let size = track!(self.value.decode(buf, eos), "tag={}", self.tag.into().0)?.0;
//         if self.value.has_item() {
//             self.is_decoding = false;
//         }
//         Ok(size)
//     }

//     fn is_decoding(&self) -> bool {
//         self.is_decoding
//     }

//     fn finish_decoding(&mut self) -> Result<Self::Item> {
//         track_assert!(!self.is_decoding, ErrorKind::InvalidInput);
//         let values = self.value.take_item().unwrap_or_else(Default::default);
//         Ok(values)
//     }
// }

// // where the key_type can be any integral or string type
// //  (so, any scalar type except for floating point types and bytes).
// #[derive(Debug, Default)]
// pub struct MapFieldDecoder<T, F, K, V>
// where
//     K: Decode,
//     V: Decode,
// {
//     inner: RepeatedFieldDecoder<
//         T,
//         F,
//         EmbeddedMessageDecoder<FieldsDecoder<(FieldDecoder<Tag1, K>, FieldDecoder<Tag2, V>)>>,
//     >,
// }
// impl<T, F, K, V> FieldDecode for MapFieldDecoder<T, F, K, V>
// where
//     T: Copy + Into<Tag>,
//     F: Default + Extend<(K::Item, V::Item)>,
//     K: Decode,
//     K::Item: Value,
//     V: Decode,
//     V::Item: Value,
// {
//     type Item = F;

//     fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
//         track!(self.inner.start_decoding(tag))
//     }

//     fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
//         track!(self.inner.field_decode(buf, eos))
//     }

//     fn is_decoding(&self) -> bool {
//         self.inner.is_decoding()
//     }

//     fn finish_decoding(&mut self) -> Result<Self::Item> {
//         track!(self.inner.finish_decoding())
//     }
// }

// #[derive(Debug)]
// pub enum OneOf2<A, B> {
//     A(A),
//     B(B),
//     None,
// }

// #[derive(Debug, Default)]
// pub struct OneOfDecoder<F> {
//     fields: F,
//     last: usize,
// }
// impl<F0, F1> FieldDecode for OneOfDecoder<(F0, F1)>
// where
//     F0: FieldDecode, // TODO: SingularFieldDecode
//     F1: FieldDecode,
// {
//     type Item = OneOf2<F0::Item, F1::Item>;

//     fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
//         if track!(self.fields.0.start_decoding(tag))? {
//             self.last = 1;
//             Ok(true)
//         } else if track!(self.fields.1.start_decoding(tag))? {
//             self.last = 2;
//             Ok(true)
//         } else {
//             Ok(false)
//         }
//     }

//     fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
//         if self.fields.0.is_decoding() {
//             track!(self.fields.0.field_decode(buf, eos))
//         } else if self.fields.1.is_decoding() {
//             track!(self.fields.1.field_decode(buf, eos))
//         } else {
//             track_panic!(ErrorKind::Other)
//         }
//     }

//     fn is_decoding(&self) -> bool {
//         self.fields.0.is_decoding() || self.fields.1.is_decoding()
//     }

//     fn finish_decoding(&mut self) -> Result<Self::Item> {
//         match self.last {
//             0 => Ok(OneOf2::None),
//             1 => track!(self.fields.0.finish_decoding()).map(OneOf2::A),
//             2 => track!(self.fields.1.finish_decoding()).map(OneOf2::B),
//             _ => unreachable!(),
//         }
//     }
// }

#[cfg(test)]
mod test {
    use bytecodec::EncodeExt;
    use bytecodec::io::{IoDecodeExt, IoEncodeExt};

    use scalar::Fixed32Encoder;
    use tag::Tag1;
    use super::*;

    macro_rules! assert_decode {
        ($decoder:ident, $value:expr, $bytes:expr) => {
            let mut decoder = $decoder::new();
            let item = track_try_unwrap!(decoder.decode_exact($bytes.as_ref()));
            assert_eq!(item, $value);
        }
    }

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
