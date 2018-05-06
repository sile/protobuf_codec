// use std::iter;
// use std::mem;
// use message::EmbeddedMessageDecoder;
// use tag::{Tag1, Tag2};
// use wire::LengthDelimitedDecoder;
use bytecodec::{ByteCount, Encode, Eos, ExactBytesEncode, Result};
// use bytecodec::combinator::{Buffered, Collect};

pub use fields::FieldsDecoder;

use message::EmbeddedMessageEncoder;
use tag::Tag;
use wire::{TagAndTypeEncoder, WireEncode, WireType};

pub trait FieldDecode {
    type Item;

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool>;
    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize>;
    fn is_decoding(&self) -> bool;
    fn finish_decoding(&mut self) -> Result<Self::Item>;
    fn requiring_bytes(&self) -> ByteCount;
}

pub trait FieldEncode: Encode {}

#[derive(Debug, Default)]
pub struct UnknownFieldDecoder {}
impl FieldDecode for UnknownFieldDecoder {
    type Item = ();

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
        panic!()
    }
    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        panic!()
    }
    fn is_decoding(&self) -> bool {
        panic!()
    }
    fn finish_decoding(&mut self) -> Result<Self::Item> {
        panic!()
    }
    fn requiring_bytes(&self) -> ByteCount {
        panic!()
    }
}

pub type MessageFieldEncoder<T, M> = FieldEncoder<T, EmbeddedMessageEncoder<M>>;

#[derive(Debug, Default)]
pub struct FieldEncoder<T, E> {
    tag: T,
    tag_and_type: TagAndTypeEncoder,
    value: E,
}
impl<T, E> FieldEncoder<T, E> {
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
    E: WireEncode,
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

// // singular: a well-formed message can have zero or one of this field (but not more than one).
// #[derive(Debug, Default)]
// pub struct FieldDecoder<T, D: Decode> {
//     tag: T,
//     value: Buffered<D>,
//     is_decoding: bool,
// }
// impl<T, D: Decode> FieldDecoder<T, D> {
//     pub fn new(tag: T, value: D) -> Self {
//         FieldDecoder {
//             tag,
//             value: value.buffered(),
//             is_decoding: false,
//         }
//     }
// }
// impl<T, D> FieldDecode for FieldDecoder<T, D>
// where
//     T: Copy + Into<Tag>,
//     D: Decode,
//     D::Item: Value,
// {
//     type Item = D::Item;

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
//         let value = self.value.take_item().unwrap_or_else(Default::default);
//         Ok(value)
//     }
// }

// #[derive(Debug, Default)]
// pub struct RepeatedFieldDecoder<T, V, D> {
//     tag: T,
//     value: D,
//     values: V,
//     is_decoding: bool,
// }
// impl<T, V, D> FieldDecode for RepeatedFieldDecoder<T, V, D>
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
//             self.is_decoding = true;
//             Ok(true)
//         }
//     }

//     fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
//         track_assert!(self.is_decoding, ErrorKind::Other);

//         let (size, item) = track!(self.value.decode(buf, eos), "tag={}", self.tag.into().0)?;
//         if let Some(value) = item {
//             self.values.extend(iter::once(value));
//             self.is_decoding = false;
//         }
//         Ok(size)
//     }

//     fn is_decoding(&self) -> bool {
//         self.is_decoding
//     }

//     fn finish_decoding(&mut self) -> Result<Self::Item> {
//         track_assert!(!self.is_decoding, ErrorKind::InvalidInput);
//         let values = mem::replace(&mut self.values, V::default());
//         Ok(values)
//     }
// }

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
