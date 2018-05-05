use bytecodec::{Eos, ErrorKind, Result};

use field::FieldDecode;
use tag::Tag;

#[derive(Debug, Default)]
pub struct FieldsDecoder<F> {
    fields: F,
}
impl<F0, F1> FieldDecode for FieldsDecoder<(F0, F1)>
where
    F0: FieldDecode,
    F1: FieldDecode,
{
    type Item = (F0::Item, F1::Item);

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if track!(self.fields.0.start_decoding(tag))? {
            Ok(true)
        } else if track!(self.fields.1.start_decoding(tag))? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        if self.fields.0.is_decoding() {
            track!(self.fields.0.field_decode(buf, eos))
        } else if self.fields.1.is_decoding() {
            track!(self.fields.1.field_decode(buf, eos))
        } else {
            track_panic!(ErrorKind::Other)
        }
    }

    fn is_decoding(&self) -> bool {
        self.fields.0.is_decoding() || self.fields.1.is_decoding()
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let v0 = track!(self.fields.0.finish_decoding())?;
        let v1 = track!(self.fields.1.finish_decoding())?;
        Ok((v0, v1))
    }
}
