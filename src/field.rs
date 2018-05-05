use std::iter;
use std::mem;
use bytecodec::{ByteCount, Decode, DecodeExt, Eos, ErrorKind, Result};
use bytecodec::combinator::{Buffered, Collect};

pub use fields::FieldsDecoder;

use {Tag, Value};
use wire::LengthDelimitedDecoder;

pub trait FieldDecode: Decode {
    fn start_decoding_field(&mut self, tag: Tag) -> Result<bool>;
    fn is_field_being_decoded(&self) -> bool;
    fn complete_field_decoding(&mut self) -> Result<Self::Item>;
}

#[derive(Debug, Default)]
pub struct FieldDecoder<T, D: Decode> {
    tag: T,
    value: Buffered<D>,
    is_running: bool,
}
impl<T, D: Decode> FieldDecoder<T, D> {
    pub fn new(tag: T, value: D) -> Self {
        FieldDecoder {
            tag,
            value: value.buffered(),
            is_running: false,
        }
    }
}
impl<T, D> Decode for FieldDecoder<T, D>
where
    T: AsRef<Tag>,
    D: Decode,
    D::Item: Value,
{
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track_assert!(self.is_running, ErrorKind::Other);

        let size = track!(self.value.decode(buf, eos), "tag={}", self.tag.as_ref().0)?.0;
        if self.value.has_item() {
            self.is_running = false;
        }
        Ok((size, None))
    }

    fn has_terminated(&self) -> bool {
        self.value.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.value.requiring_bytes()
    }
}
impl<T, D> FieldDecode for FieldDecoder<T, D>
where
    T: AsRef<Tag>,
    D: Decode,
    D::Item: Value,
{
    fn start_decoding_field(&mut self, tag: Tag) -> Result<bool> {
        if self.tag.as_ref() != &tag {
            Ok(false)
        } else {
            track_assert!(!self.is_running, ErrorKind::Other);
            track_assert!(
                self.value.has_item(),
                ErrorKind::InvalidInput,
                "This field can be appeared at most once: tag={}",
                self.tag.as_ref().0
            );
            self.is_running = true;
            Ok(true)
        }
    }

    fn is_field_being_decoded(&self) -> bool {
        self.is_running
    }

    fn complete_field_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_running, ErrorKind::InvalidInput);
        let value = self.value.take_item().unwrap_or_else(Default::default);
        Ok(value)
    }
}

#[derive(Debug, Default)]
pub struct RepeatedFieldDecoder<T, V, D> {
    tag: T,
    value: D,
    values: V,
    is_running: bool,
}
impl<T, V, D> Decode for RepeatedFieldDecoder<T, V, D>
where
    T: AsRef<Tag>,
    V: Value + Extend<D::Item>,
    D: Decode,
{
    type Item = V;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track_assert!(self.is_running, ErrorKind::Other);

        let (size, item) = track!(self.value.decode(buf, eos), "tag={}", self.tag.as_ref().0)?;
        if let Some(value) = item {
            self.values.extend(iter::once(value));
            self.is_running = false;
        }
        Ok((size, None))
    }

    fn has_terminated(&self) -> bool {
        self.value.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.value.requiring_bytes()
    }
}
impl<T, V, D> FieldDecode for RepeatedFieldDecoder<T, V, D>
where
    T: AsRef<Tag>,
    V: Value + Extend<D::Item>,
    D: Decode,
{
    fn start_decoding_field(&mut self, tag: Tag) -> Result<bool> {
        if self.tag.as_ref() != &tag {
            Ok(false)
        } else {
            track_assert!(!self.is_running, ErrorKind::Other);
            self.is_running = true;
            Ok(true)
        }
    }

    fn is_field_being_decoded(&self) -> bool {
        self.is_running
    }

    fn complete_field_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_running, ErrorKind::InvalidInput);
        let values = mem::replace(&mut self.values, V::default());
        Ok(values)
    }
}

#[derive(Debug, Default)]
pub struct PackedRepeatedFieldDecoder<T, V, D>
where
    V: Value + Extend<D::Item>,
    D: Decode,
{
    tag: T,
    value: Buffered<LengthDelimitedDecoder<Collect<D, V>>>,
    is_running: bool,
}
impl<T, V, D> Decode for PackedRepeatedFieldDecoder<T, V, D>
where
    T: AsRef<Tag>,
    V: Value + Extend<D::Item>,
    D: Decode,
{
    type Item = V;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track_assert!(self.is_running, ErrorKind::Other);

        let size = track!(self.value.decode(buf, eos), "tag={}", self.tag.as_ref().0)?.0;
        if self.value.has_item() {
            self.is_running = false;
        }
        Ok((size, None))
    }

    fn has_terminated(&self) -> bool {
        self.value.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.value.requiring_bytes()
    }
}
impl<T, V, D> FieldDecode for PackedRepeatedFieldDecoder<T, V, D>
where
    T: AsRef<Tag>,
    V: Value + Extend<D::Item>,
    D: Decode,
{
    fn start_decoding_field(&mut self, tag: Tag) -> Result<bool> {
        if self.tag.as_ref() != &tag {
            Ok(false)
        } else {
            track_assert!(!self.is_running, ErrorKind::Other);
            track_assert!(
                self.value.has_item(),
                ErrorKind::InvalidInput,
                "This field can be appeared at most once: tag={}",
                self.tag.as_ref().0
            );
            self.is_running = true;
            Ok(true)
        }
    }

    fn is_field_being_decoded(&self) -> bool {
        self.is_running
    }

    fn complete_field_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_running, ErrorKind::InvalidInput);
        let values = self.value.take_item().unwrap_or_else(Default::default);
        Ok(values)
    }
}

// TODO: map, oneof
