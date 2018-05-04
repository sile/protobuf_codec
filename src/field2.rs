use bytecodec::{ByteCount, Decode, Eos, ErrorKind, Result};

use {Tag, Value};

pub trait FieldDecode: Decode
where
    Self::Item: Value,
{
    fn start_decoding_field(&mut self, tag: Tag) -> Result<bool>;
    fn is_field_being_decoded(&self) -> bool;
    fn complete_field_decoding(&mut self) -> Result<Self::Item>;
}

#[derive(Debug, Default)]
pub struct FieldDecoder<T, V: Decode> {
    tag: T,
    value_decoder: V,
    value: Option<V::Item>,
    is_started: bool,
}
impl<T, V> Decode for FieldDecoder<T, V>
where
    T: AsRef<Tag>,
    V: Decode,
    V::Item: Value,
{
    type Item = V::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track_assert!(self.is_field_being_decoded(), ErrorKind::Other);

        let (size, item) = track!(self.value_decoder.decode(buf, eos))?;
        if let Some(value) = item {
            self.value = Some(value);
        }
        Ok((size, None))
    }

    fn has_terminated(&self) -> bool {
        self.value.is_none() && self.value_decoder.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.value.is_some() {
            ByteCount::Finite(0)
        } else {
            self.value_decoder.requiring_bytes()
        }
    }
}
impl<T, V> FieldDecode for FieldDecoder<T, V>
where
    T: AsRef<Tag>,
    V: Decode,
    V::Item: Value,
{
    fn start_decoding_field(&mut self, tag: Tag) -> Result<bool> {
        if self.tag.as_ref() != &tag {
            Ok(false)
        } else {
            track_assert!(!self.is_started, ErrorKind::Other);
            self.is_started = true;
            Ok(true)
        }
    }

    fn is_field_being_decoded(&self) -> bool {
        self.is_started && self.value.is_none()
    }

    fn complete_field_decoding(&mut self) -> Result<Self::Item> {
        if self.is_started {
            if self.value.is_none() {
                track!(self.decode(&[][..], Eos::new(true)))?;
            }
            let value = track_assert_some!(
                self.value.take(),
                ErrorKind::InvalidInput,
                "The value of the field has been decoded incompletely: tag={:?}",
                self.tag.as_ref()
            );
            self.is_started = false;
            Ok(value)
        } else {
            Ok(Default::default())
        }
    }
}
