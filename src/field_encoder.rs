use bytecodec::{ByteCount, Encode, Eos, ExactBytesEncode, Result};

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
