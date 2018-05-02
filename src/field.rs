use std::marker::PhantomData;
use bytecodec::{ByteCount, Decode, Encode, Eos, ExactBytesEncode, Result};
use bytecodec::bytes::BytesEncoder;
use bytecodec::io::IoEncodeExt;

use message::{Embedded, EmbeddedMessageEncoder, Message2, Message2Encoder};
use value::Value;
use wire::{LengthDelimited, LengthDelimitedEncoder, TagAndTypeEncoder};

pub type Tag = u32;

pub trait Field0 {
    type Value: Value;

    fn tag(&self) -> Tag;
    fn value(&self) -> Self::Value;
}

#[derive(Debug)]
pub struct Field<V> {
    pub tag: Tag,
    pub value: V,
}

#[derive(Debug, Default)]
pub struct FieldEncoder<V> {
    tag_and_type: TagAndTypeEncoder,
    value: V,
}
impl<V> Encode for FieldEncoder<V>
where
    V: Encode,
    V::Item: Value,
{
    type Item = Field<V::Item>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.tag_and_type.is_idle() {
            offset += track!(self.tag_and_type.encode(buf, eos))?;
            if !self.tag_and_type.is_idle() {
                return Ok(offset);
            }
        }

        offset += track!(self.value.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let t = (item.tag, item.value.wire_type());
        track!(self.tag_and_type.start_encoding(t))?;
        track!(self.value.start_encoding(item.value))?;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.tag_and_type.is_idle() && self.value.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.tag_and_type
            .requiring_bytes()
            .add_for_encoding(self.value.requiring_bytes())
    }
}
impl<V> ExactBytesEncode for FieldEncoder<V>
where
    V: ExactBytesEncode,
    V::Item: Value,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.tag_and_type.exact_requiring_bytes() + self.value.exact_requiring_bytes()
    }
}

#[derive(Debug)]
pub struct RepeatedFieldEncoder<I, V> {
    bytes: BytesEncoder<Vec<u8>>,
    field: FieldEncoder<V>,
    _iter: PhantomData<I>,
}
impl<I, V> Encode for RepeatedFieldEncoder<I, V>
where
    I: Iterator<Item = V::Item>,
    V: Encode,
    V::Item: Value,
{
    type Item = Field<I>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.bytes.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let tag = item.tag;
        let mut buf = Vec::new();
        for value in item.value {
            track!(self.field.start_encoding(Field { tag, value }))?;
            track!(self.field.encode_all(&mut buf))?;
        }
        track!(self.bytes.start_encoding(buf))
    }

    fn is_idle(&self) -> bool {
        self.bytes.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.bytes.requiring_bytes()
    }
}
impl<I, V> ExactBytesEncode for RepeatedFieldEncoder<I, V>
where
    I: Iterator<Item = V::Item>,
    V: Encode,
    V::Item: Value,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.bytes.exact_requiring_bytes()
    }
}
impl<I, V: Default> Default for RepeatedFieldEncoder<I, V> {
    fn default() -> Self {
        RepeatedFieldEncoder {
            bytes: BytesEncoder::default(),
            field: FieldEncoder::default(),
            _iter: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct PackedRepeatedFieldEncoder<I, V> {
    inner: FieldEncoder<LengthDelimitedEncoder<BytesEncoder<Vec<u8>>>>,
    value: V,
    _iter: PhantomData<I>,
}
impl<I, V> Encode for PackedRepeatedFieldEncoder<I, V>
where
    I: Iterator<Item = V::Item>,
    V: Encode,
{
    type Item = Field<I>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.inner.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let tag = item.tag;
        let mut buf = Vec::new();
        for value in item.value {
            track!(self.value.start_encoding(value))?;
            track!(self.value.encode_all(&mut buf))?;
        }

        let value = LengthDelimited(buf);
        track!(self.inner.start_encoding(Field { tag, value }))
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<I, V> ExactBytesEncode for PackedRepeatedFieldEncoder<I, V>
where
    I: Iterator<Item = V::Item>,
    V: Encode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
    }
}
impl<I, V: Default> Default for PackedRepeatedFieldEncoder<I, V> {
    fn default() -> Self {
        PackedRepeatedFieldEncoder {
            inner: FieldEncoder::default(),
            value: V::default(),
            _iter: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct MapFieldEncoder<I, K, V> {
    bytes: BytesEncoder<Vec<u8>>,
    field: FieldEncoder<EmbeddedMessageEncoder<Message2Encoder<K, V>>>,
    _iter: PhantomData<I>,
}
impl<I, K, V> Encode for MapFieldEncoder<I, K, V>
where
    I: Iterator<Item = (K::Item, V::Item)>,
    K: ExactBytesEncode,
    V: ExactBytesEncode,
    K::Item: Value,
    V::Item: Value,
{
    type Item = Field<I>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.bytes.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let tag = item.tag;
        let mut buf = Vec::new();
        for (k, v) in item.value {
            // TODO: let value = Embedded(Message2(Tag1(k), Tag2(v)));
            let value = Embedded(Message2(k, v));
            let field = Field { tag, value };
            track!(self.field.start_encoding(field))?;
            track!(self.field.encode_all(&mut buf))?;
        }
        track!(self.bytes.start_encoding(buf))
    }

    fn is_idle(&self) -> bool {
        self.bytes.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.bytes.requiring_bytes()
    }
}
impl<I, K, V> ExactBytesEncode for MapFieldEncoder<I, K, V>
where
    I: Iterator<Item = (K::Item, V::Item)>,
    K: ExactBytesEncode,
    V: ExactBytesEncode,
    K::Item: Value,
    V::Item: Value,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.bytes.exact_requiring_bytes()
    }
}
impl<I, K: Default, V: Default> Default for MapFieldEncoder<I, K, V> {
    fn default() -> Self {
        MapFieldEncoder {
            bytes: Default::default(),
            field: Default::default(),
            _iter: PhantomData,
        }
    }
}

#[derive(Debug)]
pub enum OneOf2<A, B> {
    A(A),
    B(B),
}
