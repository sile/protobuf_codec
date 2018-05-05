use std::iter;
use std::mem;
use bytecodec::{Decode, DecodeExt, Eos, ErrorKind, Result};
use bytecodec::combinator::{Buffered, Collect};

pub use fields::FieldsDecoder;

use {Tag, Value};
use wire::LengthDelimitedDecoder;

pub trait FieldDecode {
    type Item;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool>;
    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize>;
    fn is_decoding(&self) -> bool;
    fn finish_decoding(&mut self) -> Result<Self::Item>;
}

#[derive(Debug, Default)]
pub struct FieldDecoder<T, D: Decode> {
    tag: T,
    value: Buffered<D>,
    is_decoding: bool,
}
impl<T, D: Decode> FieldDecoder<T, D> {
    pub fn new(tag: T, value: D) -> Self {
        FieldDecoder {
            tag,
            value: value.buffered(),
            is_decoding: false,
        }
    }
}
impl<T, D> FieldDecode for FieldDecoder<T, D>
where
    T: AsRef<Tag>,
    D: Decode,
    D::Item: Value,
{
    type Item = D::Item;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.tag.as_ref() != &tag {
            Ok(false)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track_assert!(
                self.value.has_item(),
                ErrorKind::InvalidInput,
                "This field can be appeared at most once: tag={}",
                self.tag.as_ref().0
            );
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track_assert!(self.is_decoding, ErrorKind::Other);

        let size = track!(self.value.decode(buf, eos), "tag={}", self.tag.as_ref().0)?.0;
        if self.value.has_item() {
            self.is_decoding = false;
        }
        Ok(size)
    }

    fn is_decoding(&self) -> bool {
        self.is_decoding
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding, ErrorKind::InvalidInput);
        let value = self.value.take_item().unwrap_or_else(Default::default);
        Ok(value)
    }
}

#[derive(Debug, Default)]
pub struct RepeatedFieldDecoder<T, V, D> {
    tag: T,
    value: D,
    values: V,
    is_decoding: bool,
}
impl<T, V, D> FieldDecode for RepeatedFieldDecoder<T, V, D>
where
    T: AsRef<Tag>,
    V: Value + Extend<D::Item>,
    D: Decode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.tag.as_ref() != &tag {
            Ok(false)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track_assert!(self.is_decoding, ErrorKind::Other);

        let (size, item) = track!(self.value.decode(buf, eos), "tag={}", self.tag.as_ref().0)?;
        if let Some(value) = item {
            self.values.extend(iter::once(value));
            self.is_decoding = false;
        }
        Ok(size)
    }

    fn is_decoding(&self) -> bool {
        self.is_decoding
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding, ErrorKind::InvalidInput);
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
    is_decoding: bool,
}
impl<T, V, D> FieldDecode for PackedRepeatedFieldDecoder<T, V, D>
where
    T: AsRef<Tag>,
    V: Value + Extend<D::Item>,
    D: Decode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.tag.as_ref() != &tag {
            Ok(false)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track_assert!(
                self.value.has_item(),
                ErrorKind::InvalidInput,
                "This field can be appeared at most once: tag={}",
                self.tag.as_ref().0
            );
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track_assert!(self.is_decoding, ErrorKind::Other);

        let size = track!(self.value.decode(buf, eos), "tag={}", self.tag.as_ref().0)?.0;
        if self.value.has_item() {
            self.is_decoding = false;
        }
        Ok(size)
    }

    fn is_decoding(&self) -> bool {
        self.is_decoding
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding, ErrorKind::InvalidInput);
        let values = self.value.take_item().unwrap_or_else(Default::default);
        Ok(values)
    }
}

// // TODO: map, oneof
