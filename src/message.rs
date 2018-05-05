use bytecodec::{ByteCount, Decode, Eos, Result};

use field::FieldDecode;
// use value::Value;
use wire::{LengthDelimitedDecoder, TagAndTypeDecoder};

pub trait Message {}

// #[derive(Debug, Default)]
// pub struct Embedded<M>(pub M);
// impl<M: Message + Default> Value for Embedded<M> {
//     fn wire_type(&self) -> WireType {
//         WireType::LengthDelimited
//     }
// }

#[derive(Debug, Default)]
pub struct MessageDecoder<F> {
    tag_and_type: TagAndTypeDecoder,
    field: F,
}
impl<F: FieldDecode> Decode for MessageDecoder<F> {
    type Item = F::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let mut offset = 0;
        while offset < buf.len() {
            if self.field.is_decoding() {
                let size = track!(self.field.field_decode(&buf[offset..], eos))?;
                offset += size;
                if self.field.is_decoding() {
                    return Ok((offset, None));
                }
            } else {
                let (size, item) = track!(self.tag_and_type.decode(&buf[offset..], eos))?;
                offset += size;
                if let Some((tag, _wire_type)) = item {
                    if !track!(self.field.start_decoding(tag))? {
                        // TODO: ignore the field
                    }
                }
            }
        }
        if eos.is_reached() {
            track!(self.tag_and_type.decode(&[][..], eos))?;
            let v = track!(self.field.finish_decoding())?;
            Ok((offset, Some(v)))
        } else {
            Ok((offset, None))
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        // TODO:
        ByteCount::Unknown
    }
}

#[derive(Debug, Default)]
pub struct EmbeddedMessageDecoder<F>(LengthDelimitedDecoder<MessageDecoder<F>>);
impl<F: FieldDecode> Decode for EmbeddedMessageDecoder<F> {
    type Item = F::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track!(self.0.decode(buf, eos))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}

// #[derive(Debug, Default)]
// pub struct EmbeddedMessageEncoder<E>(LengthDelimitedEncoder<E>);
// impl<E> Encode for EmbeddedMessageEncoder<E>
// where
//     E: ExactBytesEncode,
//     E::Item: Message,
// {
//     type Item = Embedded<E::Item>;

//     fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
//         track!(self.0.encode(buf, eos))
//     }

//     fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
//         track!(self.0.start_encoding(LengthDelimited(item.0)))
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

// // TODO: SparseMessage2

// #[derive(Debug)]
// pub struct Message2<F1, F2>(pub F1, pub F2);
// impl<F1, F2> Message for Message2<F1, F2>
// where
//     F1: Value,
//     F2: Value,
// {
// }

// #[derive(Debug, Default)]
// pub struct Message2Encoder<F1, F2> {
//     field1: FieldEncoder<F1>,
//     field2: FieldEncoder<F2>,
// }
// impl<F1, F2> Encode for Message2Encoder<F1, F2>
// where
//     F1: Encode,
//     F2: Encode,
//     F1::Item: Value,
//     F2::Item: Value,
// {
//     type Item = Message2<F1::Item, F2::Item>;

//     fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
//         let mut offset = 0;
//         try_encode!(self.field1, offset, buf, eos);
//         offset += track!(self.field2.encode(&mut buf[offset..], eos))?;
//         Ok(offset)
//     }

//     fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
//         track!(self.field1.start_encoding(Field {
//             tag: 1,
//             value: item.0,
//         }))?;
//         track!(self.field2.start_encoding(Field {
//             tag: 2,
//             value: item.1,
//         }))?;
//         Ok(())
//     }

//     fn is_idle(&self) -> bool {
//         self.field1.is_idle() && self.field2.is_idle()
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         self.field1
//             .requiring_bytes()
//             .add_for_encoding(self.field2.requiring_bytes())
//     }
// }
// impl<F1, F2> ExactBytesEncode for Message2Encoder<F1, F2>
// where
//     F1: ExactBytesEncode,
//     F2: ExactBytesEncode,
//     F1::Item: Value,
//     F2::Item: Value,
// {
//     fn exact_requiring_bytes(&self) -> u64 {
//         self.field1.exact_requiring_bytes() + self.field2.exact_requiring_bytes()
//     }
// }
