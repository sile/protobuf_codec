use bytecodec::{ByteCount, Decode, Eos, Result};

use field::FieldDecode;
use tag::Tag;

#[derive(Debug, Default)]
pub struct FieldsDecoder<F> {
    fields: F,
}
impl<F0, F1> Decode for FieldsDecoder<(F0, F1)>
where
    F0: FieldDecode,
    F1: FieldDecode,
{
    type Item = (F0::Item, F1::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let size = if self.fields.0.is_field_being_decoded() {
            track!(self.fields.0.decode(buf, eos))?.0
        } else if self.fields.1.is_field_being_decoded() {
            track!(self.fields.1.decode(buf, eos))?.0
        } else {
            0
        };
        Ok((size, None))
    }

    fn has_terminated(&self) -> bool {
        self.fields.0.has_terminated() || self.fields.1.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.fields
            .0
            .requiring_bytes()
            .add_for_decoding(self.fields.1.requiring_bytes())
    }
}
impl<F0, F1> FieldDecode for FieldsDecoder<(F0, F1)>
where
    F0: FieldDecode,
    F1: FieldDecode,
{
    fn start_decoding_field(&mut self, tag: Tag) -> Result<bool> {
        if track!(self.fields.0.start_decoding_field(tag))? {
            Ok(true)
        } else if track!(self.fields.1.start_decoding_field(tag))? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn is_field_being_decoded(&self) -> bool {
        self.fields.0.is_field_being_decoded() || self.fields.1.is_field_being_decoded()
    }

    fn complete_field_decoding(&mut self) -> Result<Self::Item> {
        let v0 = track!(self.fields.0.complete_field_decoding())?;
        let v1 = track!(self.fields.1.complete_field_decoding())?;
        Ok((v0, v1))
    }
}
