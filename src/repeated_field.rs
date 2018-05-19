use bytecodec::combinator::Collect;
use bytecodec::{ByteCount, Decode, DecodeExt, Encode, Eos, ErrorKind, Result, SizedEncode};
use std::fmt;
use std::iter;
use std::mem;

use field::num::{F1, F2, FieldNum};
use field::{FieldDecode, FieldDecoder, FieldEncode, FieldEncoder, Fields,
            RepeatedMessageFieldDecoder, RepeatedMessageFieldEncoder};
use message::{EmbeddedMessageDecoder, EmbeddedMessageEncoder, MessageDecoder, MessageEncoder};
use scalar::BytesEncoder;
use value::{MapKeyDecode, MapKeyEncode, NumericValueDecode, NumericValueEncode, ValueDecode,
            ValueEncode};
use wire::{LengthDelimitedDecoder, Tag, TagEncoder, WireType};

/// Decoder for repeated fields.
#[derive(Debug, Default)]
pub struct RepeatedFieldDecoder<F, V, D> {
    num: F,
    decoder: D,
    values: V,
    is_decoding: bool,
}
impl<F, V: Default, D> RepeatedFieldDecoder<F, V, D> {
    /// Makes a new `RepeatedFieldDecoder` instance.
    pub fn new(field_num: F, value_decoder: D) -> Self {
        RepeatedFieldDecoder {
            num: field_num,
            decoder: value_decoder,
            values: V::default(),
            is_decoding: false,
        }
    }
}
impl<F, V, D> FieldDecode for RepeatedFieldDecoder<F, V, D>
where
    F: Copy + Into<FieldNum>,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
    D: ValueDecode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.num.into() == tag.field_num {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track_assert_eq!(self.decoder.wire_type(), tag.wire_type, ErrorKind::InvalidInput; tag);
            self.is_decoding = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let size = track!(self.decoder.decode(buf, eos); self.num.into())?;
        if self.decoder.is_idle() {
            let value = track!(self.decoder.finish_decoding())?;
            self.is_decoding = false;
            self.values.extend(iter::once(value));
        }
        Ok(size)
    }

    fn is_decoding(&self) -> bool {
        self.is_decoding
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding(), ErrorKind::Other);
        Ok(mem::replace(&mut self.values, V::default()))
    }

    fn requiring_bytes(&self) -> ByteCount {
        if !self.is_decoding {
            ByteCount::Finite(0)
        } else {
            self.decoder.requiring_bytes()
        }
    }
}

/// Encoder for repeated fields.
#[derive(Debug)]
pub struct RepeatedFieldEncoder<F, V: IntoIterator, E> {
    field: FieldEncoder<F, E>,
    values: Option<V::IntoIter>,
}
impl<F, V: IntoIterator, E: ValueEncode> RepeatedFieldEncoder<F, V, E> {
    /// Makes new `RepeatedFieldEncoder` instance.
    pub fn new(field_num: F, value_encoder: E) -> Self {
        RepeatedFieldEncoder {
            field: FieldEncoder::new(field_num, value_encoder),
            values: None,
        }
    }
}
impl<F: Default, V: IntoIterator, E: Default> Default for RepeatedFieldEncoder<F, V, E> {
    fn default() -> Self {
        RepeatedFieldEncoder {
            field: FieldEncoder::default(),
            values: None,
        }
    }
}
impl<F, E, V> Encode for RepeatedFieldEncoder<F, V, E>
where
    F: Copy + Into<FieldNum>,
    V: IntoIterator<Item = E::Item>,
    E: ValueEncode,
{
    type Item = V;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        while offset < buf.len() {
            if self.field.is_idle() {
                if let Some(item) = self.values.as_mut().and_then(|x| x.next()) {
                    track!(self.field.start_encoding(item))?;
                } else {
                    self.values = None;
                    break;
                }
            }
            bytecodec_try_encode!(self.field, offset, buf, eos);
        }
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        self.values = Some(item.into_iter());
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.values.is_none()
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_idle() {
            ByteCount::Finite(0)
        } else {
            ByteCount::Unknown
        }
    }
}
impl<F, V, E> FieldEncode for RepeatedFieldEncoder<F, V, E>
where
    F: Copy + Into<FieldNum>,
    V: IntoIterator<Item = E::Item>,
    E: ValueEncode,
{
}

/// Decoder for packed repeated fields.
///
/// Ordinarily, it is recommended to use `RepeatedNumericFieldDecoder` instead of this.
#[derive(Debug, Default)]
pub struct PackedRepeatedFieldDecoder<F, V, D>
where
    V: Default + Extend<D::Item>,
    D: NumericValueDecode,
{
    num: F,
    decoder: LengthDelimitedDecoder<Collect<D, V>>,
    values: V,
    is_decoding: bool,
}
impl<F, V, D> PackedRepeatedFieldDecoder<F, V, D>
where
    V: Default + Extend<D::Item>,
    D: NumericValueDecode,
{
    /// Makes a new `PackedRepeatedFieldDecoder` instance.
    pub fn new(field_num: F, value_decoder: D) -> Self {
        PackedRepeatedFieldDecoder {
            num: field_num,
            decoder: LengthDelimitedDecoder::new(value_decoder.collect()),
            values: V::default(),
            is_decoding: false,
        }
    }

    fn inner_mut(&mut self) -> &mut D {
        self.decoder.inner_mut().inner_mut()
    }
}
impl<F, V, D> FieldDecode for PackedRepeatedFieldDecoder<F, V, D>
where
    F: Copy + Into<FieldNum>,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
    D: NumericValueDecode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.num.into() != tag.field_num {
            Ok(false)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track_assert!(self.is_decoding, ErrorKind::Other);

        let size = track!(self.decoder.decode(buf, eos); self.num.into())?;
        if self.decoder.is_idle() {
            let values = track!(self.decoder.finish_decoding())?;
            self.values.extend(values.into_iter());
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

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_decoding {
            self.decoder.requiring_bytes()
        } else {
            ByteCount::Finite(0)
        }
    }
}

/// Decoder for repeated numeric fields.
///
/// This can decode numeric fields regardless of whether they are packed or not.
#[derive(Debug, Default)]
pub struct RepeatedNumericFieldDecoder<F, V, D>
where
    V: Default + Extend<D::Item>,
    D: NumericValueDecode,
{
    decoder: PackedRepeatedFieldDecoder<F, V, D>,
    is_decoding: bool,
}
impl<F, V, D> RepeatedNumericFieldDecoder<F, V, D>
where
    V: Default + Extend<D::Item>,
    D: NumericValueDecode,
{
    /// Makes a new `RepeatedNumericFieldDecoder` instance.
    pub fn new(field_num: F, value_decoder: D) -> Self {
        RepeatedNumericFieldDecoder {
            decoder: PackedRepeatedFieldDecoder::new(field_num, value_decoder),
            is_decoding: false,
        }
    }
}
impl<F, V, D> FieldDecode for RepeatedNumericFieldDecoder<F, V, D>
where
    F: Copy + Into<FieldNum>,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
    D: NumericValueDecode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.decoder.num.into() != tag.field_num {
            Ok(false)
        } else if tag.wire_type == WireType::LengthDelimited {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track!(self.decoder.start_decoding(tag))?;
            self.is_decoding = true;
            Ok(true)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        if self.decoder.is_decoding {
            let size = track!(self.decoder.field_decode(buf, eos))?;
            if !self.decoder.is_decoding {
                self.is_decoding = false;
            }
            Ok(size)
        } else {
            track_assert!(self.is_decoding, ErrorKind::Other);
            let size = track!(self.decoder.inner_mut().decode(buf, eos); self.decoder.num.into())?;
            if self.decoder.inner_mut().is_idle() {
                let value = track!(self.decoder.inner_mut().finish_decoding())?;
                self.decoder.values.extend(iter::once(value));
                self.is_decoding = false;
            }
            Ok(size)
        }
    }

    fn is_decoding(&self) -> bool {
        self.is_decoding
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding, ErrorKind::InvalidInput);
        track!(self.decoder.finish_decoding())
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_decoding {
            self.decoder.requiring_bytes()
        } else {
            ByteCount::Finite(0)
        }
    }
}

type MapMessageDecoder<K, V> = MessageDecoder<Fields<(FieldDecoder<F1, K>, FieldDecoder<F2, V>)>>;

/// Decoder for map fields.
#[derive(Default)]
pub struct MapFieldDecoder<F, M, K, V>
where
    K: MapKeyDecode,
    V: ValueDecode,
{
    inner: RepeatedMessageFieldDecoder<F, M, MapMessageDecoder<K, V>>,
}
impl<F, M: Default, K: MapKeyDecode, V: ValueDecode> MapFieldDecoder<F, M, K, V> {
    /// Makes a new `MapFieldDecoder` instance.
    pub fn new(field_num: F, key_decoder: K, value_decoder: V) -> Self {
        let fields = Fields::new((
            FieldDecoder::new(F1, key_decoder),
            FieldDecoder::new(F2, value_decoder),
        ));
        let message = MessageDecoder::new(fields);
        let inner =
            RepeatedMessageFieldDecoder::new(field_num, EmbeddedMessageDecoder::new(message));
        MapFieldDecoder { inner }
    }
}
impl<F, M, K, V> FieldDecode for MapFieldDecoder<F, M, K, V>
where
    F: Copy + Into<FieldNum>,
    M: Default + Extend<(K::Item, V::Item)> + IntoIterator<Item = (K::Item, V::Item)>,
    K: MapKeyDecode,
    V: ValueDecode,
{
    type Item = M;

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

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<F, M, K, V> fmt::Debug for MapFieldDecoder<F, M, K, V>
where
    K: MapKeyDecode,
    V: ValueDecode,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapFieldDecoder {{ .. }}")
    }
}

/// Encoder for packed repeated fields.
#[derive(Debug)]
pub struct PackedRepeatedFieldEncoder<F, V: IntoIterator, E> {
    num: F,
    values: Option<V::IntoIter>,
    tag: TagEncoder,
    value: E,
    bytes: BytesEncoder,
}
impl<F, V: IntoIterator, E> PackedRepeatedFieldEncoder<F, V, E> {
    /// Makes a new `PackedRepeatedFieldEncoder` instance.
    pub fn new(field_num: F, value_encoder: E) -> Self {
        PackedRepeatedFieldEncoder {
            num: field_num,
            values: None,
            tag: TagEncoder::new(),
            value: value_encoder,
            bytes: BytesEncoder::new(),
        }
    }
}
impl<F: Default, V: IntoIterator, E: Default> Default for PackedRepeatedFieldEncoder<F, V, E> {
    fn default() -> Self {
        PackedRepeatedFieldEncoder {
            num: F::default(),
            values: None,
            tag: TagEncoder::new(),
            value: E::default(),
            bytes: BytesEncoder::default(),
        }
    }
}
impl<F, V, E> Encode for PackedRepeatedFieldEncoder<F, V, E>
where
    F: Copy + Into<FieldNum>,
    V: IntoIterator<Item = E::Item>,
    E: NumericValueEncode,
{
    type Item = V;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.tag, offset, buf, eos);
        bytecodec_try_encode!(self.bytes, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let tag = Tag::from((self.num.into(), self.bytes.wire_type()));
        track!(self.tag.start_encoding(tag))?;
        let mut buf = Vec::new();
        for v in item {
            track!(self.value.start_encoding(v))?;

            let eos = Eos::new(false);
            let old_len = buf.len();
            let new_len = old_len + self.value.exact_requiring_bytes() as usize;
            buf.resize(new_len, 0);
            track!(self.value.encode(&mut buf[old_len..], eos))?;
        }
        track!(self.bytes.start_encoding(buf))?;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.bytes.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }
}
impl<F, V, E> SizedEncode for PackedRepeatedFieldEncoder<F, V, E>
where
    F: Copy + Into<FieldNum>,
    V: IntoIterator<Item = E::Item>,
    E: NumericValueEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.tag.exact_requiring_bytes() + self.bytes.exact_requiring_bytes()
    }
}
impl<F, V, E> FieldEncode for PackedRepeatedFieldEncoder<F, V, E>
where
    F: Copy + Into<FieldNum>,
    V: IntoIterator<Item = E::Item>,
    E: NumericValueEncode,
{
}

type MapMessageEncoder<K, V> = MessageEncoder<Fields<(FieldEncoder<F1, K>, FieldEncoder<F2, V>)>>;

/// Encoder for map fields.
#[derive(Default)]
pub struct MapFieldEncoder<F, M: IntoIterator, K, V> {
    inner: RepeatedMessageFieldEncoder<F, M, MapMessageEncoder<K, V>>,
}
impl<F, M, K, V> MapFieldEncoder<F, M, K, V>
where
    F: Copy + Into<FieldNum>,
    M: IntoIterator<Item = (K::Item, V::Item)>,
    K: SizedEncode + MapKeyEncode,
    V: SizedEncode + ValueEncode,
{
    /// Makes a new `MapFieldEncoder` instance.
    pub fn new(field_num: F, key_encoder: K, value_encoder: V) -> Self {
        let fields = Fields::new((
            FieldEncoder::new(F1, key_encoder),
            FieldEncoder::new(F2, value_encoder),
        ));
        let message = MessageEncoder::new(fields);
        let inner =
            RepeatedMessageFieldEncoder::new(field_num, EmbeddedMessageEncoder::new(message));
        MapFieldEncoder { inner }
    }
}
impl<F, M, K, V> Encode for MapFieldEncoder<F, M, K, V>
where
    F: Copy + Into<FieldNum>,
    M: IntoIterator<Item = (K::Item, V::Item)>,
    K: SizedEncode + MapKeyEncode,
    V: SizedEncode + ValueEncode,
{
    type Item = M;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.inner.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.inner.start_encoding(item))
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<F, M, K, V> FieldEncode for MapFieldEncoder<F, M, K, V>
where
    F: Copy + Into<FieldNum>,
    M: IntoIterator<Item = (K::Item, V::Item)>,
    K: SizedEncode + MapKeyEncode,
    V: SizedEncode + ValueEncode,
{
}
impl<F, M: IntoIterator, K, V> fmt::Debug for MapFieldEncoder<F, M, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapFieldEncoder {{ .. }}")
    }
}
