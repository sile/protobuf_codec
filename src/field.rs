use std::mem;
use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, ExactBytesEncode, Result};
use bytecodec::bytes::CopyableBytesDecoder;
use bytecodec::combinator::SkipRemaining;
use bytecodec::value::NullDecoder;

// pub use fields::{FieldsDecoder, FieldsEncoder};
// pub use oneof::{OneOf2, OneOf3, OneOf4, OneOf5, OneOf6, OneOf7, OneOf8};
// pub use repeated_field::{MapFieldDecoder, MapFieldEncoder, PackedRepeatedFieldDecoder,
//                          PackedRepeatedFieldEncoder, RepeatedFieldDecoder, RepeatedFieldEncoder,
//                          RepeatedNumericFieldDecoder};

// use message::{EmbeddedMessageDecoder, EmbeddedMessageEncoder};
use tag::Tag;
use wire::{LengthDelimitedDecoder, TagAndTypeEncoder, VarintDecoder, WireType};

pub trait ValueDecode: Decode {
    fn wire_type(&self) -> WireType;
    fn merge_values(old: Self::Item, new: Self::Item) -> Self::Item;
}

pub trait OptionalValueDecode: ValueDecode {
    type Optional: Default;
    fn merge_optional_values(old: Self::Optional, new: Self::Optional) -> Self::Optional;
}

pub trait ValueEncode: Encode {
    fn wire_type(&self) -> WireType;
}

pub trait OptionalValueEncode: ValueEncode {
    type Optional;
    fn is_none_value(value: &Self::Optional) -> bool;
}

pub trait MapKeyDecode: ValueDecode {}

pub trait MapKeyEncode: ValueEncode {}

pub trait NumericDecode: ValueDecode {}

pub trait NumericEncode: ValueEncode {}

// pub trait FieldDecode {
//     type Item;

//     fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool>;
//     fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize>;
//     fn is_decoding(&self) -> bool;
//     fn finish_decoding(&mut self) -> Result<Self::Item>;
//     fn requiring_bytes(&self) -> ByteCount;
//     fn merge(&self, old: Self::Item, new: Self::Item) -> Self::Item;
// }

// pub trait SingularFieldDecode: FieldDecode {}

// pub trait FieldEncode: Encode {}

// pub trait SingularFieldEncode: FieldEncode {}

// pub type MessageFieldDecoder<T, M> = FieldDecoder<T, EmbeddedMessageDecoder<M>>;

// #[derive(Debug, Default)]
// pub struct FieldDecoder<T, D: WireDecode> {
//     tag: T,
//     decoder: D,
//     value: D::Value,
//     is_decoding: bool,
// }
// impl<T, D: WireDecode> FieldDecoder<T, D> {
//     pub fn new(tag: T, value_decoder: D) -> Self {
//         FieldDecoder {
//             tag,
//             decoder: value_decoder,
//             value: D::Value::default(),
//             is_decoding: false,
//         }
//     }
// }
// impl<T, D> FieldDecode for FieldDecoder<T, D>
// where
//     T: Copy + Into<Tag>,
//     D: WireDecode,
// {
//     type Item = D::Value;

//     fn start_decoding(&mut self, tag: Tag, _: WireType) -> Result<bool> {
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

//         let (size, item) = track!(self.decoder.decode(buf, eos); self.tag.into())?;
//         if let Some(new) = item {
//             self.is_decoding = false;
//             self.value = self.decoder.merge(
//                 mem::replace(&mut self.value, D::Value::default()),
//                 new.into(),
//             );
//         }
//         Ok(size)
//     }

//     fn is_decoding(&self) -> bool {
//         self.is_decoding
//     }

//     fn finish_decoding(&mut self) -> Result<Self::Item> {
//         track_assert!(!self.is_decoding, ErrorKind::InvalidInput);
//         let value = mem::replace(&mut self.value, D::Value::default());
//         Ok(value)
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         if self.is_decoding {
//             self.decoder.requiring_bytes()
//         } else {
//             ByteCount::Unknown
//         }
//     }

//     fn merge(&self, old: Self::Item, new: Self::Item) -> Self::Item {
//         self.decoder.merge(old, new)
//     }
// }
// impl<T, D> SingularFieldDecode for FieldDecoder<T, D>
// where
//     T: Copy + Into<Tag>,
//     D: WireDecode,
// {
// }

// #[derive(Debug)]
// pub struct UnknownFieldDecoder(UnknownFieldDecoderInner);
// impl FieldDecode for UnknownFieldDecoder {
//     type Item = ();

//     fn start_decoding(&mut self, _tag: Tag, wire_type: WireType) -> Result<bool> {
//         track_assert!(!self.is_decoding(), ErrorKind::Other);
//         self.0 = match wire_type {
//             WireType::Varint => UnknownFieldDecoderInner::Varint(Default::default()),
//             WireType::Bit32 => UnknownFieldDecoderInner::Bit32(Default::default()),
//             WireType::Bit64 => UnknownFieldDecoderInner::Bit64(Default::default()),
//             WireType::LengthDelimited => {
//                 UnknownFieldDecoderInner::LengthDelimited(Default::default())
//             }
//         };
//         Ok(true)
//     }

//     fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
//         let (size, decoded) = match self.0 {
//             UnknownFieldDecoderInner::None => (0, false),
//             UnknownFieldDecoderInner::Varint(ref mut d) => {
//                 track!(d.decode(buf, eos)).map(|(n, i)| (n, i.is_some()))?
//             }
//             UnknownFieldDecoderInner::Bit32(ref mut d) => {
//                 track!(d.decode(buf, eos)).map(|(n, i)| (n, i.is_some()))?
//             }
//             UnknownFieldDecoderInner::Bit64(ref mut d) => {
//                 track!(d.decode(buf, eos)).map(|(n, i)| (n, i.is_some()))?
//             }
//             UnknownFieldDecoderInner::LengthDelimited(ref mut d) => {
//                 track!(d.decode(buf, eos)).map(|(n, i)| (n, i.is_some()))?
//             }
//         };
//         if decoded {
//             self.0 = UnknownFieldDecoderInner::None;
//         }
//         Ok(size)
//     }

//     fn is_decoding(&self) -> bool {
//         if let UnknownFieldDecoderInner::None = self.0 {
//             false
//         } else {
//             true
//         }
//     }

//     fn finish_decoding(&mut self) -> Result<Self::Item> {
//         track_assert!(!self.is_decoding(), ErrorKind::Other);
//         Ok(())
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         match self.0 {
//             UnknownFieldDecoderInner::None => ByteCount::Unknown,
//             UnknownFieldDecoderInner::Varint(ref d) => d.requiring_bytes(),
//             UnknownFieldDecoderInner::Bit32(ref d) => d.requiring_bytes(),
//             UnknownFieldDecoderInner::Bit64(ref d) => d.requiring_bytes(),
//             UnknownFieldDecoderInner::LengthDelimited(ref d) => d.requiring_bytes(),
//         }
//     }

//     fn merge(&self, (): Self::Item, (): Self::Item) -> Self::Item {
//         ()
//     }
// }
// impl Default for UnknownFieldDecoder {
//     fn default() -> Self {
//         UnknownFieldDecoder(UnknownFieldDecoderInner::None)
//     }
// }

// #[derive(Debug)]
// enum UnknownFieldDecoderInner {
//     None,
//     Varint(VarintDecoder),
//     Bit32(CopyableBytesDecoder<[u8; 4]>),
//     Bit64(CopyableBytesDecoder<[u8; 8]>),
//     LengthDelimited(LengthDelimitedDecoder<SkipRemaining<NullDecoder>>),
// }

// pub type MessageFieldEncoder<T, M> = FieldEncoder<T, EmbeddedMessageEncoder<M>>;

// #[derive(Debug, Default)]
// pub struct FieldEncoder<T, E> {
//     tag: T,
//     tag_and_type: TagAndTypeEncoder,
//     value: E,
// }
// impl<T, E> FieldEncoder<T, E>
// where
//     T: Copy + Into<Tag>,
//     E: WireEncode,
// {
//     pub fn new(tag: T, value_encoder: E) -> Self {
//         FieldEncoder {
//             tag,
//             tag_and_type: TagAndTypeEncoder::new(),
//             value: value_encoder,
//         }
//     }

//     pub(crate) fn force_start_encoding(&mut self, item: E::Item) -> Result<()> {
//         let tag_and_type = (self.tag.into(), self.value.wire_type());
//         track!(self.tag_and_type.start_encoding(tag_and_type))?;
//         track!(self.value.start_encoding(item))?;
//         Ok(())
//     }
// }
// impl<T, E> Encode for FieldEncoder<T, E>
// where
//     T: Copy + Into<Tag>,
//     E: WireEncode,
// {
//     type Item = E::Value;

//     fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
//         let mut offset = 0;
//         bytecodec_try_encode!(self.tag_and_type, offset, buf, eos);
//         bytecodec_try_encode!(self.value, offset, buf, eos);
//         Ok(offset)
//     }

//     fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
//         track!(self.value.start_encoding_value(item))?;
//         if !self.value.is_idle() {
//             let tag_and_type = (self.tag.into(), self.value.wire_type());
//             track!(self.tag_and_type.start_encoding(tag_and_type))?;
//         }
//         Ok(())
//     }

//     fn is_idle(&self) -> bool {
//         self.value.is_idle()
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         self.tag_and_type
//             .requiring_bytes()
//             .add_for_encoding(self.value.requiring_bytes())
//     }
// }
// impl<T, E> ExactBytesEncode for FieldEncoder<T, E>
// where
//     T: Copy + Into<Tag>,
//     E: ExactBytesEncode + WireEncode,
// {
//     fn exact_requiring_bytes(&self) -> u64 {
//         self.tag_and_type.exact_requiring_bytes() + self.value.exact_requiring_bytes()
//     }
// }
// impl<T, E> FieldEncode for FieldEncoder<T, E>
// where
//     T: Copy + Into<Tag>,
//     E: WireEncode,
// {
// }
// impl<T, E> SingularFieldEncode for FieldEncoder<T, E>
// where
//     T: Copy + Into<Tag>,
//     E: WireEncode,
// {
// }

// #[cfg(test)]
// mod test {
//     use bytecodec::EncodeExt;
//     use bytecodec::io::{IoDecodeExt, IoEncodeExt};

//     use scalar::Fixed32Encoder;
//     use tag::Tag1;
//     use super::*;

//     macro_rules! assert_decode {
//         ($decoder:ident, $value:expr, $bytes:expr) => {
//             let mut decoder = $decoder::new();
//             let item = track_try_unwrap!(decoder.decode_exact($bytes.as_ref()));
//             assert_eq!(item, $value);
//         }
//     }

//     macro_rules! assert_encode {
//         ($encoder:ty, $value:expr, $bytes:expr) => {
//             let mut buf = Vec::new();
//             let mut encoder: $encoder = track_try_unwrap!(EncodeExt::with_item($value));
//             track_try_unwrap!(encoder.encode_all(&mut buf));
//             assert_eq!(buf, $bytes);
//         }
//     }

//     #[test]
//     fn field_encoder_works() {
//         assert_encode!(FieldEncoder<Tag1, Fixed32Encoder>, 123, [0x0d, 0x7b, 0x00, 0x00, 0x00]);
//     }
// }
