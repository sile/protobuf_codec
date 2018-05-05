use std::iter;
use std::mem;
use bytecodec::{Decode, DecodeExt, Eos, ErrorKind, Result};
use bytecodec::combinator::{Buffered, Collect};

pub use fields::FieldsDecoder;

use {Tag, Value};
use message::EmbeddedMessageDecoder;
use tag::{Tag1, Tag2};
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
    T: Copy + Into<Tag>,
    D: Decode,
    D::Item: Value,
{
    type Item = D::Item;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.tag.into() != tag {
            Ok(false)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track_assert!(
                self.value.has_item(),
                ErrorKind::InvalidInput,
                "This field can be appeared at most once: tag={}",
                self.tag.into().0
            );
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track_assert!(self.is_decoding, ErrorKind::Other);

        let size = track!(self.value.decode(buf, eos), "tag={}", self.tag.into().0)?.0;
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
    T: Copy + Into<Tag>,
    V: Default + Extend<D::Item>,
    D: Decode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.tag.into() != tag {
            Ok(false)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track_assert!(self.is_decoding, ErrorKind::Other);

        let (size, item) = track!(self.value.decode(buf, eos), "tag={}", self.tag.into().0)?;
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
    V: Default + Extend<D::Item>,
    D: Decode,
{
    tag: T,
    value: Buffered<LengthDelimitedDecoder<Collect<D, V>>>,
    is_decoding: bool,
}
impl<T, V, D> FieldDecode for PackedRepeatedFieldDecoder<T, V, D>
where
    T: Copy + Into<Tag>,
    V: Default + Extend<D::Item>,
    D: Decode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.tag.into() != tag {
            Ok(false)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track_assert!(
                self.value.has_item(),
                ErrorKind::InvalidInput,
                "This field can be appeared at most once: tag={}",
                self.tag.into().0
            );
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track_assert!(self.is_decoding, ErrorKind::Other);

        let size = track!(self.value.decode(buf, eos), "tag={}", self.tag.into().0)?.0;
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

#[derive(Debug, Default)]
pub struct MapFieldDecoder<T, F, K, V>
where
    K: Decode,
    V: Decode,
{
    inner: RepeatedFieldDecoder<
        T,
        F,
        EmbeddedMessageDecoder<FieldsDecoder<(FieldDecoder<Tag1, K>, FieldDecoder<Tag2, V>)>>,
    >,
}
impl<T, F, K, V> FieldDecode for MapFieldDecoder<T, F, K, V>
where
    T: Copy + Into<Tag>,
    F: Default + Extend<(K::Item, V::Item)>,
    K: Decode,
    K::Item: Value,
    V: Decode,
    V::Item: Value,
{
    type Item = F;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        track!(self.inner.start_decoding(tag))
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.inner.field_decode(buf, eos))
    }

    fn is_decoding(&self) -> bool {
        self.inner.is_decoding()
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track!(self.inner.finish_decoding())
    }
}

#[derive(Debug)]
pub enum OneOf2<A, B> {
    A(A),
    B(B),
    None,
}

#[derive(Debug, Default)]
pub struct OneOfDecoder<F> {
    fields: F,
    last: usize,
}
impl<F0, F1> FieldDecode for OneOfDecoder<(F0, F1)>
where
    F0: FieldDecode, // TODO: SingularFieldDecode
    F1: FieldDecode,
{
    type Item = OneOf2<F0::Item, F1::Item>;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if track!(self.fields.0.start_decoding(tag))? {
            self.last = 1;
            Ok(true)
        } else if track!(self.fields.1.start_decoding(tag))? {
            self.last = 2;
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
        match self.last {
            0 => Ok(OneOf2::None),
            1 => track!(self.fields.0.finish_decoding()).map(OneOf2::A),
            2 => track!(self.fields.1.finish_decoding()).map(OneOf2::B),
            _ => unreachable!(),
        }
    }
}
