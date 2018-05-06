use bytecodec::{ByteCount, Encode, Eos, ExactBytesEncode, Result};

use tag::Tag;
use wire::{TagAndTypeEncoder, WireEncode};

pub trait FieldEncode: Encode {}

#[derive(Debug, Default)]
pub struct FieldsEncoder<F> {
    fields: F,
}
impl<F0, F1> Encode for FieldsEncoder<(F0, F1)>
where
    F0: Encode,
    F1: Encode,
{
    type Item = (F0::Item, F1::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.fields.0, offset, buf, eos);
        bytecodec_try_encode!(self.fields.1, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.fields.0.start_encoding(item.0))?;
        track!(self.fields.1.start_encoding(item.1))?;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.fields.1.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.fields
            .0
            .requiring_bytes()
            .add_for_encoding(self.fields.1.requiring_bytes())
    }
}
impl<F0, F1> ExactBytesEncode for FieldsEncoder<(F0, F1)>
where
    F0: ExactBytesEncode,
    F1: ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.fields.0.exact_requiring_bytes() + self.fields.1.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
pub struct FieldEncoder<T, E> {
    tag: T,
    tag_and_type_encoder: TagAndTypeEncoder,
    field_value_encoder: E,
}
impl<T, E> Encode for FieldEncoder<T, E>
where
    T: Clone + Into<Tag>,
    E: WireEncode,
{
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.tag_and_type_encoder, offset, buf, eos);
        bytecodec_try_encode!(self.field_value_encoder, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let tag_and_type = (
            self.tag.clone().into(),
            self.field_value_encoder.wire_type(),
        );
        track!(self.tag_and_type_encoder.start_encoding(tag_and_type))?;
        track!(self.field_value_encoder.start_encoding(item))?;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.field_value_encoder.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.tag_and_type_encoder
            .requiring_bytes()
            .add_for_encoding(self.field_value_encoder.requiring_bytes())
    }
}
impl<T, E> ExactBytesEncode for FieldEncoder<T, E>
where
    T: Clone + Into<Tag>,
    E: ExactBytesEncode + WireEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.tag_and_type_encoder.exact_requiring_bytes()
            + self.field_value_encoder.exact_requiring_bytes()
    }
}

// // TODO: default
// #[derive(Debug, Default)]
// pub struct RepeatedFieldEncoder<T, F: IntoIterator, E> {
//     inner: FieldEncoder<T, E>,
//     field_values: Option<F::IntoIter>,
// }
// impl<T, F, E> Encode for RepeatedFieldEncoder<T, F, E>
// where
//     T: Clone + Into<Tag>,
//     F: IntoIterator,
//     E: Encode<Item = F::Item>,
//     E::Item: Value,
// {
//     type Item = F;

//     fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
//         let mut offset = 0;
//         while offset < buf.len() {
//             if self.inner.is_idle() {
//                 if let Some(item) = self.field_values.as_mut().and_then(|x| x.next()) {
//                     track!(self.inner.start_encoding(item))?;
//                 } else {
//                     self.field_values = None;
//                     break;
//                 }
//             }
//             bytecodec_try_encode!(self.inner, offset, buf, eos);
//         }
//         Ok(offset)
//     }

//     fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
//         track_assert!(self.is_idle(), ErrorKind::EncoderFull);
//         self.field_values = Some(item.into_iter());
//         Ok(())
//     }

//     fn is_idle(&self) -> bool {
//         self.field_values.is_none()
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         if self.is_idle() {
//             ByteCount::Finite(0)
//         } else {
//             ByteCount::Unknown
//         }
//     }
// }

// // TODO: default
// #[derive(Debug, Default)]
// pub struct PackedRepeatedFieldEncoder<T, F: IntoIterator, E> {
//     tag: T,
//     tag_and_type_encoder: TagAndTypeEncoder,
//     field_value_encoder: E,
//     field_values: Option<F::IntoIter>,
// }
// impl<T, F, E> Encode for PackedRepeatedFieldEncoder<T, F, E>
// where
//     T: Clone + Into<Tag>,
//     F: IntoIterator,
//     E: Encode<Item = F::Item>,
//     E::Item: Value, // TODO: FixedLengthValue
// {
//     type Item = F;

//     fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
//         let mut offset = 0;
//         bytecodec_try_encode!(self.tag_and_type_encoder, offset, buf, eos);

//         while offset < buf.len() {
//             if self.field_value_encoder.is_idle() {
//                 if let Some(item) = self.field_values.as_mut().and_then(|x| x.next()) {
//                     track!(self.field_value_encoder.start_encoding(item))?;
//                 } else {
//                     self.field_values = None;
//                     break;
//                 }
//             }
//             bytecodec_try_encode!(self.field_value_encoder, offset, buf, eos);
//         }
//         Ok(offset)
//     }

//     fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
//         track_assert!(self.is_idle(), ErrorKind::EncoderFull);
//         let tag_and_type = (self.tag.clone().into(), E::Item::default().wire_type());
//         track!(self.tag_and_type_encoder.start_encoding(tag_and_type))?;
//         self.field_values = Some(item.into_iter());
//         Ok(())
//     }

//     fn is_idle(&self) -> bool {
//         self.field_values.is_none()
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         if self.is_idle() {
//             ByteCount::Finite(0)
//         } else {
//             ByteCount::Unknown
//         }
//     }
// }
