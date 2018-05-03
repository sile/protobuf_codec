use std::iter;
use std::marker::PhantomData;
use std::mem;
use bytecodec::{ByteCount, Decode, Encode, Eos, ExactBytesEncode, Result};
use bytecodec::combinator::Collect;
// use bytecodec::bytes::BytesEncoder;
// use bytecodec::io::IoEncodeExt;

// use message::{Embedded, EmbeddedMessageEncoder, Message2, Message2Encoder};
use value::Value;
use wire::{LengthDelimited, LengthDelimitedDecoder, LengthDelimitedEncoder, TagAndTypeEncoder};

pub type Tag = u32;

pub trait Field: Default {
    type Value: Value;
    fn tag(&self) -> Tag;
    // TODO: wire_type (?)
    fn from_value(value: Self::Value) -> Self;
    fn into_value(self) -> Self::Value;
    fn value_mut(&mut self) -> &mut Self::Value;
}

pub trait FieldDecode: Decode {
    fn start_decoding_field(&mut self, tag: Tag) -> bool;
    fn is_suspended(&self) -> bool;
}

#[derive(Debug, Default)]
pub struct FieldDecoder<F, V> {
    field: F,
    value: V,
}
impl<F, V> Decode for FieldDecoder<F, V>
where
    F: Field,
    V: Decode<Item = F::Value>,
{
    type Item = F;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let (size, item) = track!(self.value.decode(buf, eos))?;
        let item = item.map(F::from_value);
        Ok((size, item))
    }

    fn has_terminated(&self) -> bool {
        self.value.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.value.requiring_bytes()
    }
}
impl<F, V> FieldDecode for FieldDecoder<F, V>
where
    F: Field,
    V: Decode<Item = F::Value>,
{
    fn start_decoding_field(&mut self, tag: Tag) -> bool {
        self.field.tag() == tag
    }

    fn is_suspended(&self) -> bool {
        false
    }
}

#[derive(Debug, Default)]
pub struct RepeatedFieldDecoder<F, D> {
    field: F,
    value_decoder: D,
    is_suspended: bool,
}
impl<F, D> Decode for RepeatedFieldDecoder<F, D>
where
    F: Field,
    D: Decode,
    F::Value: Extend<D::Item>,
{
    type Item = F;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let (size, item) = track!(self.value_decoder.decode(buf, eos))?;
        if let Some(v) = item {
            self.is_suspended = true;
            self.field.value_mut().extend(iter::once(v));
        }
        if eos.is_reached() {
            let field = mem::replace(&mut self.field, F::default());
            Ok((size, Some(field)))
        } else {
            Ok((size, None))
        }
    }

    fn has_terminated(&self) -> bool {
        self.value_decoder.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.value_decoder.requiring_bytes()
    }
}
impl<F, D> FieldDecode for RepeatedFieldDecoder<F, D>
where
    F: Field,
    D: Decode,
    F::Value: Extend<D::Item>,
{
    fn start_decoding_field(&mut self, tag: Tag) -> bool {
        self.is_suspended = false;
        self.field.tag() == tag
    }

    fn is_suspended(&self) -> bool {
        self.is_suspended
    }
}

#[derive(Debug, Default)]
pub struct PackedRepeatedFieldDecoder<F: Field, D> {
    field: F,
    value_decoder: LengthDelimitedDecoder<Collect<D, F::Value>>,
}
impl<F, D> Decode for PackedRepeatedFieldDecoder<F, D>
where
    F: Field,
    D: Decode,
    F::Value: Extend<D::Item> + Default,
{
    type Item = F;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let (size, item) = track!(self.value_decoder.decode(buf, eos))?;
        let item = item.map(F::from_value);
        Ok((size, item))
    }

    fn has_terminated(&self) -> bool {
        self.value_decoder.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.value_decoder.requiring_bytes()
    }
}
impl<F, D> FieldDecode for PackedRepeatedFieldDecoder<F, D>
where
    F: Field,
    D: Decode,
    F::Value: Extend<D::Item> + Default,
{
    fn start_decoding_field(&mut self, tag: Tag) -> bool {
        self.field.tag() == tag
    }

    fn is_suspended(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct FieldEncoder<F, V> {
    tag_and_type: TagAndTypeEncoder,
    value: V,
    _field: PhantomData<F>,
}
impl<F, V> Encode for FieldEncoder<F, V>
where
    F: Field,
    V: Encode<Item = F::Value>,
{
    type Item = F;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        try_encode!(self.tag_and_type, offset, buf, eos);
        offset += track!(self.value.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let tag = item.tag();
        let value = item.into_value();
        let t = (tag, value.wire_type());
        track!(self.tag_and_type.start_encoding(t))?;
        track!(self.value.start_encoding(value))?;
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
impl<F, V> ExactBytesEncode for FieldEncoder<F, V>
where
    F: Field,
    V: ExactBytesEncode<Item = F::Value>,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.tag_and_type.exact_requiring_bytes() + self.value.exact_requiring_bytes()
    }
}
impl<F, V: Default> Default for FieldEncoder<F, V> {
    fn default() -> Self {
        FieldEncoder {
            tag_and_type: Default::default(),
            value: Default::default(),
            _field: PhantomData,
        }
    }
}

// #[derive(Debug)]
// pub struct RepeatedFieldEncoder<I, V> {
//     bytes: BytesEncoder<Vec<u8>>,
//     field: FieldEncoder<V>,
//     _iter: PhantomData<I>,
// }
// impl<I, V> Encode for RepeatedFieldEncoder<I, V>
// where
//     I: Iterator<Item = V::Item>,
//     V: Encode,
//     V::Item: Value,
// {
//     type Item = Field<I>;

//     fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
//         track!(self.bytes.encode(buf, eos))
//     }

//     fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
//         let tag = item.tag;
//         let mut buf = Vec::new();
//         for value in item.value {
//             track!(self.field.start_encoding(Field { tag, value }))?;
//             track!(self.field.encode_all(&mut buf))?;
//         }
//         track!(self.bytes.start_encoding(buf))
//     }

//     fn is_idle(&self) -> bool {
//         self.bytes.is_idle()
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         self.bytes.requiring_bytes()
//     }
// }
// impl<I, V> ExactBytesEncode for RepeatedFieldEncoder<I, V>
// where
//     I: Iterator<Item = V::Item>,
//     V: Encode,
//     V::Item: Value,
// {
//     fn exact_requiring_bytes(&self) -> u64 {
//         self.bytes.exact_requiring_bytes()
//     }
// }
// impl<I, V: Default> Default for RepeatedFieldEncoder<I, V> {
//     fn default() -> Self {
//         RepeatedFieldEncoder {
//             bytes: BytesEncoder::default(),
//             field: FieldEncoder::default(),
//             _iter: PhantomData,
//         }
//     }
// }

// #[derive(Debug)]
// pub struct PackedRepeatedFieldEncoder<I, V> {
//     inner: FieldEncoder<LengthDelimitedEncoder<BytesEncoder<Vec<u8>>>>,
//     value: V,
//     _iter: PhantomData<I>,
// }
// impl<I, V> Encode for PackedRepeatedFieldEncoder<I, V>
// where
//     I: Iterator<Item = V::Item>,
//     V: Encode,
// {
//     type Item = Field<I>;

//     fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
//         track!(self.inner.encode(buf, eos))
//     }

//     fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
//         let tag = item.tag;
//         let mut buf = Vec::new();
//         for value in item.value {
//             track!(self.value.start_encoding(value))?;
//             track!(self.value.encode_all(&mut buf))?;
//         }

//         let value = LengthDelimited(buf);
//         track!(self.inner.start_encoding(Field { tag, value }))
//     }

//     fn is_idle(&self) -> bool {
//         self.inner.is_idle()
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         self.inner.requiring_bytes()
//     }
// }
// impl<I, V> ExactBytesEncode for PackedRepeatedFieldEncoder<I, V>
// where
//     I: Iterator<Item = V::Item>,
//     V: Encode,
// {
//     fn exact_requiring_bytes(&self) -> u64 {
//         self.inner.exact_requiring_bytes()
//     }
// }
// impl<I, V: Default> Default for PackedRepeatedFieldEncoder<I, V> {
//     fn default() -> Self {
//         PackedRepeatedFieldEncoder {
//             inner: FieldEncoder::default(),
//             value: V::default(),
//             _iter: PhantomData,
//         }
//     }
// }

// #[derive(Debug)]
// pub struct MapFieldEncoder<I, K, V> {
//     bytes: BytesEncoder<Vec<u8>>,
//     field: FieldEncoder<EmbeddedMessageEncoder<Message2Encoder<K, V>>>,
//     _iter: PhantomData<I>,
// }
// impl<I, K, V> Encode for MapFieldEncoder<I, K, V>
// where
//     I: Iterator<Item = (K::Item, V::Item)>,
//     K: ExactBytesEncode,
//     V: ExactBytesEncode,
//     K::Item: Value,
//     V::Item: Value,
// {
//     type Item = Field<I>;

//     fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
//         track!(self.bytes.encode(buf, eos))
//     }

//     fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
//         let tag = item.tag;
//         let mut buf = Vec::new();
//         for (k, v) in item.value {
//             // TODO: let value = Embedded(Message2(Tag1(k), Tag2(v)));
//             let value = Embedded(Message2(k, v));
//             let field = Field { tag, value };
//             track!(self.field.start_encoding(field))?;
//             track!(self.field.encode_all(&mut buf))?;
//         }
//         track!(self.bytes.start_encoding(buf))
//     }

//     fn is_idle(&self) -> bool {
//         self.bytes.is_idle()
//     }

//     fn requiring_bytes(&self) -> ByteCount {
//         self.bytes.requiring_bytes()
//     }
// }
// impl<I, K, V> ExactBytesEncode for MapFieldEncoder<I, K, V>
// where
//     I: Iterator<Item = (K::Item, V::Item)>,
//     K: ExactBytesEncode,
//     V: ExactBytesEncode,
//     K::Item: Value,
//     V::Item: Value,
// {
//     fn exact_requiring_bytes(&self) -> u64 {
//         self.bytes.exact_requiring_bytes()
//     }
// }
// impl<I, K: Default, V: Default> Default for MapFieldEncoder<I, K, V> {
//     fn default() -> Self {
//         MapFieldEncoder {
//             bytes: Default::default(),
//             field: Default::default(),
//             _iter: PhantomData,
//         }
//     }
// }

// #[derive(Debug)]
// pub enum OneOf2<A, B> {
//     A(A),
//     B(B),
// }
