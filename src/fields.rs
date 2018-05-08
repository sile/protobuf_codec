use bytecodec::{ByteCount, Encode, Eos, ErrorKind, ExactBytesEncode, Result};

use field::{FieldDecode, FieldEncode};
use tag::Tag;
use wire::WireType;

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

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
        if track!(self.fields.0.start_decoding(tag, wire_type))? {
            Ok(true)
        } else if track!(self.fields.1.start_decoding(tag, wire_type))? {
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

    fn requiring_bytes(&self) -> ByteCount {
        if self.fields.0.is_decoding() {
            self.fields.0.requiring_bytes()
        } else if self.fields.1.is_decoding() {
            self.fields.1.requiring_bytes()
        } else {
            ByteCount::Unknown
        }
    }

    fn merge(&self, old: Self::Item, new: Self::Item) -> Self::Item {
        let v0 = self.fields.0.merge(old.0, new.0);
        let v1 = self.fields.1.merge(old.1, new.1);
        (v0, v1)
    }
}

#[derive(Debug, Default)]
pub struct FieldsEncoder<F> {
    fields: F,
}
impl<F0, F1> Encode for FieldsEncoder<(F0, F1)>
where
    F0: FieldEncode,
    F1: FieldEncode,
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
    F0: FieldEncode + ExactBytesEncode,
    F1: FieldEncode + ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.fields.0.exact_requiring_bytes() + self.fields.1.exact_requiring_bytes()
    }
}
impl<F0, F1> FieldEncode for FieldsEncoder<(F0, F1)>
where
    F0: FieldEncode,
    F1: FieldEncode,
{
}
