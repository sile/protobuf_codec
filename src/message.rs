// use bytecodec::{ByteCount, Decode, Encode, Eos, ExactBytesEncode, Result};

// use field::FieldDecode;
// // use value::Value;
// use wire::{LengthDelimitedDecoder, LengthDelimitedEncoder, TagAndTypeDecoder};

// pub trait Message {}

// // #[derive(Debug, Default)]
// // pub struct Embedded<M>(pub M);
// // impl<M: Message + Default> Value for Embedded<M> {
// //     fn wire_type(&self) -> WireType {
// //         WireType::LengthDelimited
// //     }
// // }

// // TODO: TopLevelMessageDecoder or Dispatcher
// #[derive(Debug, Default)]
// pub struct MessageDecoder<F> {
//     tag_and_type: TagAndTypeDecoder,
//     field: F,
// }
// impl<F: FieldDecode> Decode for MessageDecoder<F> {
//     type Item = F::Item;

//     fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
//         let mut offset = 0;
//         while offset < buf.len() {
//             if self.field.is_decoding() {
//                 let size = track!(self.field.field_decode(&buf[offset..], eos))?;
//                 offset += size;
//                 if self.field.is_decoding() {
//                     return Ok((offset, None));
//                 }
//             } else {
//                 let (size, item) = track!(self.tag_and_type.decode(&buf[offset..], eos))?;
//                 offset += size;
//                 if let Some((tag, _wire_type)) = item {
//                     if !track!(self.field.start_decoding(tag))? {
//                         // TODO: ignore the field
//                     }
//                 }
//             }
//         }
//         if eos.is_reached() {
//             track!(self.tag_and_type.decode(&[][..], eos))?;
//             let v = track!(self.field.finish_decoding())?;
//             Ok((offset, Some(v)))
//         } else {
//             Ok((offset, None))
//         }
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         // TODO:
//         ByteCount::Unknown
//     }
// }

// #[derive(Debug, Default)]
// pub struct EmbeddedMessageDecoder<F>(LengthDelimitedDecoder<MessageDecoder<F>>);
// impl<F: FieldDecode> Decode for EmbeddedMessageDecoder<F> {
//     type Item = F::Item;

//     fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
//         track!(self.0.decode(buf, eos))
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         self.0.requiring_bytes()
//     }
// }

// #[derive(Debug, Default)]
// pub struct EmbeddedMessageEncoder<E>(LengthDelimitedEncoder<E>);
// impl<E> Encode for EmbeddedMessageEncoder<E>
// where
//     E: ExactBytesEncode,
// {
//     type Item = E::Item;

//     fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
//         track!(self.0.encode(buf, eos))
//     }

//     fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
//         track!(self.0.start_encoding(item))
//     }

//     fn is_idle(&self) -> bool {
//         self.0.is_idle()
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         self.0.requiring_bytes()
//     }
// }
// impl<E> ExactBytesEncode for EmbeddedMessageEncoder<E>
// where
//     E: ExactBytesEncode,
//     E::Item: Message,
// {
//     fn exact_requiring_bytes(&self) -> u64 {
//         self.0.exact_requiring_bytes()
//     }
// }
