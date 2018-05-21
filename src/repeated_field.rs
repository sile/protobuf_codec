use bytecodec::combinator::Collect;
use bytecodec::{ByteCount, Decode, DecodeExt, Encode, Eos, ErrorKind, Result, SizedEncode};
use std::fmt;
use std::iter;
use std::mem;

use field::num::{F1, F2, FieldNum};
use field::{FieldDecode, FieldDecoder, FieldEncode, FieldEncoder, Fields, MessageFieldDecoder,
            MessageFieldEncoder, RequiredFieldDecode, RequiredFieldEncode};
use message::{MessageDecode, MessageDecoder, MessageEncode, MessageEncoder};
use scalar::BytesEncoder;
use value::{MapKeyDecode, MapKeyEncode, NumericValueDecode, NumericValueEncode, ValueDecode,
            ValueEncode};
use wire::{LengthDelimitedDecoder, Tag, TagEncoder, WireType};

/// Decoder and encoder for repeated fields.
#[derive(Debug)]
pub struct Repeated<T, V: IntoIterator> {
    inner: T,
    values: Option<V>,
    value_iter: Option<V::IntoIter>,
}
impl<T, V: IntoIterator> Repeated<T, V> {
    /// Makes a new `Repeated` instance.
    pub fn new(inner: T) -> Self {
        Repeated {
            inner,
            values: None,
            value_iter: None,
        }
    }

    /// Returns a reference to the inner field encoder/decoder.
    pub fn inner_ref(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the inner field encoder/decoder.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Takes the ownership of the instance, and returns the inner field encoder/decoder.
    pub fn into_inner(self) -> T {
        self.inner
    }
}
impl<T: Default, V: IntoIterator> Default for Repeated<T, V> {
    fn default() -> Self {
        Repeated {
            inner: T::default(),
            values: None,
            value_iter: None,
        }
    }
}
impl<D, V> Decode for Repeated<D, V>
where
    D: RequiredFieldDecode,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
{
    type Item = V;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let size = track!(self.inner.decode(buf, eos))?;
        if self.inner.is_idle() {
            let value = track!(self.inner.finish_decoding())?;
            let values = track_assert_some!(self.values.as_mut(), ErrorKind::InconsistentState);
            values.extend(iter::once(value));
        }
        Ok(size)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.inner.is_present(), ErrorKind::IncompleteDecoding);
        let values = track_assert_some!(self.values.take(), ErrorKind::InconsistentState);
        Ok(values)
    }

    fn is_idle(&self) -> bool {
        !self.inner.is_present()
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_idle() {
            ByteCount::Finite(0)
        } else {
            self.inner.requiring_bytes()
        }
    }
}
impl<D, V> FieldDecode for Repeated<D, V>
where
    D: RequiredFieldDecode,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
{
    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        let started = track!(self.inner.start_decoding(tag))?;
        if started && self.values.is_none() {
            self.values = Some(V::default());
        }
        Ok(started)
    }
}
impl<E, V> Encode for Repeated<E, V>
where
    E: RequiredFieldEncode,
    V: IntoIterator<Item = E::Item>,
{
    type Item = V;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        while offset < buf.len() {
            if self.inner.is_idle() {
                if let Some(item) = self.value_iter.as_mut().and_then(|x| x.next()) {
                    track!(self.inner.start_encoding(item))?;
                } else {
                    self.value_iter = None;
                    break;
                }
            }
            bytecodec_try_encode!(self.inner, offset, buf, eos);
        }
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        self.value_iter = Some(item.into_iter());
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.value_iter.is_none()
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_idle() {
            ByteCount::Finite(0)
        } else {
            ByteCount::Unknown
        }
    }
}
impl<E, V> FieldEncode for Repeated<E, V>
where
    E: RequiredFieldEncode,
    V: IntoIterator<Item = E::Item>,
{
}

/// Decoder for packed repeated fields.
///
/// Actually this can decode fields regardless of whether they are packed or not.
#[derive(Debug, Default)]
pub struct PackedFieldDecoder<F, D, V>
where
    D: NumericValueDecode,
    V: Default + Extend<D::Item>,
{
    num: F,
    decoder: LengthDelimitedDecoder<Collect<D, V>>,
    values: V,
    is_packed: bool,
    is_decoding: bool,
}
impl<F, D, V> PackedFieldDecoder<F, D, V>
where
    D: NumericValueDecode,
    V: Default + Extend<D::Item>,
{
    /// Makes a new `PackedFieldDecoder` instance.
    pub fn new(field_num: F, value_decoder: D) -> Self {
        PackedFieldDecoder {
            num: field_num,
            decoder: LengthDelimitedDecoder::new(value_decoder.collect()),
            values: V::default(),
            is_packed: false,
            is_decoding: false,
        }
    }
}
impl<F, D, V> Decode for PackedFieldDecoder<F, D, V>
where
    F: Copy + Into<FieldNum>,
    D: NumericValueDecode,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
{
    type Item = V;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        if !self.is_decoding {
            return Ok(0);
        }

        let mut size = 0;
        if self.is_packed {
            bytecodec_try_decode!(self.decoder, size, buf, eos; self.num.into());
            self.values.extend(track!(self.decoder.finish_decoding())?);
        } else {
            bytecodec_try_decode!(self.decoder.inner_mut().inner_mut(), size, buf, eos; self.num.into());
            let v = track!(self.decoder.inner_mut().inner_mut().finish_decoding())?;
            self.values.extend(iter::once(v));
        }
        self.is_decoding = false;
        Ok(size)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.is_decoding, ErrorKind::IncompleteDecoding);
        Ok(mem::replace(&mut self.values, V::default()))
    }

    fn is_idle(&self) -> bool {
        !self.is_decoding
    }

    fn requiring_bytes(&self) -> ByteCount {
        if !self.is_decoding {
            ByteCount::Finite(0)
        } else if self.is_packed {
            self.decoder.requiring_bytes()
        } else {
            self.decoder.inner_ref().inner_ref().requiring_bytes()
        }
    }
}
impl<F, D, V> FieldDecode for PackedFieldDecoder<F, D, V>
where
    F: Copy + Into<FieldNum>,
    D: NumericValueDecode,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
{
    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.num.into() != tag.field_num {
            Ok(false)
        } else if tag.wire_type == WireType::LengthDelimited {
            track_assert!(!self.is_decoding, ErrorKind::InconsistentState);
            self.is_packed = true;
            self.is_decoding = true;
            Ok(true)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::InconsistentState);
            self.is_packed = false;
            self.is_decoding = true;
            Ok(true)
        }
    }
}

type ScalarEntryDecoder<K, V> = MessageDecoder<Fields<(FieldDecoder<F1, K>, FieldDecoder<F2, V>)>>;
type MessageEntryDecoder<K, V> =
    MessageDecoder<Fields<(FieldDecoder<F1, K>, MessageFieldDecoder<F2, V>)>>;

/// Decoder for map fields which have scalar values.
#[derive(Default)]
pub struct MapFieldDecoder<F, K, V, M>
where
    M: IntoIterator<Item = (K::Item, V::Item)>,
    K: MapKeyDecode,
    V: ValueDecode,
{
    inner: Repeated<MessageFieldDecoder<F, ScalarEntryDecoder<K, V>>, M>,
}
impl<F, K, V, M> MapFieldDecoder<F, K, V, M>
where
    K: MapKeyDecode,
    V: ValueDecode,
    M: IntoIterator<Item = (K::Item, V::Item)> + Default,
{
    /// Makes a new `MapFieldDecoder` instance.
    pub fn new(field_num: F, key_decoder: K, value_decoder: V) -> Self {
        let fields = Fields::new((
            FieldDecoder::new(F1, key_decoder),
            FieldDecoder::new(F2, value_decoder),
        ));
        let message = MessageDecoder::new(fields);
        let inner = Repeated::new(MessageFieldDecoder::new(field_num, message));
        MapFieldDecoder { inner }
    }
}
impl<F, K, V, M> Decode for MapFieldDecoder<F, K, V, M>
where
    F: Copy + Into<FieldNum>,
    K: MapKeyDecode,
    V: ValueDecode,
    M: Default + Extend<(K::Item, V::Item)> + IntoIterator<Item = (K::Item, V::Item)>,
{
    type Item = M;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.inner.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track!(self.inner.finish_decoding())
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<F, K, V, M> FieldDecode for MapFieldDecoder<F, K, V, M>
where
    F: Copy + Into<FieldNum>,
    K: MapKeyDecode,
    V: ValueDecode,
    M: Default + Extend<(K::Item, V::Item)> + IntoIterator<Item = (K::Item, V::Item)>,
{
    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        track!(self.inner.start_decoding(tag))
    }
}
impl<F, K, V, M> fmt::Debug for MapFieldDecoder<F, K, V, M>
where
    K: MapKeyDecode,
    V: ValueDecode,
    M: IntoIterator<Item = (K::Item, V::Item)>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapFieldDecoder {{ .. }}")
    }
}

/// Decoder for map fields which have message values.
#[derive(Default)]
pub struct MapMessageFieldDecoder<F, K, V, M>
where
    M: IntoIterator<Item = (K::Item, V::Item)>,
    K: MapKeyDecode,
    V: MessageDecode,
{
    inner: Repeated<MessageFieldDecoder<F, MessageEntryDecoder<K, V>>, M>,
}
impl<F, K, V, M> MapMessageFieldDecoder<F, K, V, M>
where
    K: MapKeyDecode,
    V: MessageDecode,
    M: IntoIterator<Item = (K::Item, V::Item)> + Default,
{
    /// Makes a new `MapMessageFieldDecoder` instance.
    pub fn new(field_num: F, key_decoder: K, value_decoder: V) -> Self {
        let fields = Fields::new((
            FieldDecoder::new(F1, key_decoder),
            MessageFieldDecoder::new(F2, value_decoder),
        ));
        let message = MessageDecoder::new(fields);
        let inner = Repeated::new(MessageFieldDecoder::new(field_num, message));
        MapMessageFieldDecoder { inner }
    }
}
impl<F, K, V, M> Decode for MapMessageFieldDecoder<F, K, V, M>
where
    F: Copy + Into<FieldNum>,
    K: MapKeyDecode,
    V: MessageDecode,
    M: Default + Extend<(K::Item, V::Item)> + IntoIterator<Item = (K::Item, V::Item)>,
{
    type Item = M;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.inner.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track!(self.inner.finish_decoding())
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<F, K, V, M> FieldDecode for MapMessageFieldDecoder<F, K, V, M>
where
    F: Copy + Into<FieldNum>,
    K: MapKeyDecode,
    V: MessageDecode,
    M: Default + Extend<(K::Item, V::Item)> + IntoIterator<Item = (K::Item, V::Item)>,
{
    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        track!(self.inner.start_decoding(tag))
    }
}
impl<F, K, V, M> fmt::Debug for MapMessageFieldDecoder<F, K, V, M>
where
    K: MapKeyDecode,
    V: MessageDecode,
    M: IntoIterator<Item = (K::Item, V::Item)>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapMessageFieldDecoder {{ .. }}")
    }
}

/// Encoder for packed repeated fields.
#[derive(Debug)]
pub struct PackedFieldEncoder<F, E, V: IntoIterator> {
    num: F,
    values: Option<V::IntoIter>,
    tag: TagEncoder,
    value: E,
    bytes: BytesEncoder,
}
impl<F, E, V: IntoIterator> PackedFieldEncoder<F, E, V> {
    /// Makes a new `PackedFieldEncoder` instance.
    pub fn new(field_num: F, value_encoder: E) -> Self {
        PackedFieldEncoder {
            num: field_num,
            values: None,
            tag: TagEncoder::new(),
            value: value_encoder,
            bytes: BytesEncoder::new(),
        }
    }
}
impl<F: Default, E: Default, V: IntoIterator> Default for PackedFieldEncoder<F, E, V> {
    fn default() -> Self {
        PackedFieldEncoder {
            num: F::default(),
            values: None,
            tag: TagEncoder::new(),
            value: E::default(),
            bytes: BytesEncoder::default(),
        }
    }
}
impl<F, E, V> Encode for PackedFieldEncoder<F, E, V>
where
    F: Copy + Into<FieldNum>,
    E: NumericValueEncode,
    V: IntoIterator<Item = E::Item>,
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
impl<F, E, V> SizedEncode for PackedFieldEncoder<F, E, V>
where
    F: Copy + Into<FieldNum>,
    E: NumericValueEncode,
    V: IntoIterator<Item = E::Item>,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.tag.exact_requiring_bytes() + self.bytes.exact_requiring_bytes()
    }
}
impl<F, E, V> FieldEncode for PackedFieldEncoder<F, E, V>
where
    F: Copy + Into<FieldNum>,
    E: NumericValueEncode,
    V: IntoIterator<Item = E::Item>,
{
}

type ScalarEntryEncoder<K, V> = MessageEncoder<Fields<(FieldEncoder<F1, K>, FieldEncoder<F2, V>)>>;
type MessageEntryEncoder<K, V> =
    MessageEncoder<Fields<(FieldEncoder<F1, K>, MessageFieldEncoder<F2, V>)>>;

/// Encoder for map fields which have scalar values.
#[derive(Default)]
pub struct MapFieldEncoder<F, K, V, M: IntoIterator> {
    inner: Repeated<MessageFieldEncoder<F, ScalarEntryEncoder<K, V>>, M>,
}
impl<F, K, V, M> MapFieldEncoder<F, K, V, M>
where
    F: Copy + Into<FieldNum>,
    K: SizedEncode + MapKeyEncode,
    V: SizedEncode + ValueEncode,
    M: IntoIterator<Item = (K::Item, V::Item)>,
{
    /// Makes a new `MapFieldEncoder` instance.
    pub fn new(field_num: F, key_encoder: K, value_encoder: V) -> Self {
        let fields = Fields::new((
            FieldEncoder::new(F1, key_encoder),
            FieldEncoder::new(F2, value_encoder),
        ));
        let message = MessageEncoder::new(fields);
        let inner = Repeated::new(MessageFieldEncoder::new(field_num, message));
        MapFieldEncoder { inner }
    }
}
impl<F, K, V, M> Encode for MapFieldEncoder<F, K, V, M>
where
    F: Copy + Into<FieldNum>,
    K: SizedEncode + MapKeyEncode,
    V: SizedEncode + ValueEncode,
    M: IntoIterator<Item = (K::Item, V::Item)>,
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
impl<F, K, V, M> FieldEncode for MapFieldEncoder<F, K, V, M>
where
    F: Copy + Into<FieldNum>,
    K: SizedEncode + MapKeyEncode,
    V: SizedEncode + ValueEncode,
    M: IntoIterator<Item = (K::Item, V::Item)>,
{
}
impl<F, K, V, M: IntoIterator> fmt::Debug for MapFieldEncoder<F, K, V, M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapFieldEncoder {{ .. }}")
    }
}

/// Encoder for map fields which have message values.
#[derive(Default)]
pub struct MapMessageFieldEncoder<F, K, V, M: IntoIterator> {
    inner: Repeated<MessageFieldEncoder<F, MessageEntryEncoder<K, V>>, M>,
}
impl<F, K, V, M> MapMessageFieldEncoder<F, K, V, M>
where
    F: Copy + Into<FieldNum>,
    K: SizedEncode + MapKeyEncode,
    V: SizedEncode + MessageEncode,
    M: IntoIterator<Item = (K::Item, V::Item)>,
{
    /// Makes a new `MapMessageFieldEncoder` instance.
    pub fn new(field_num: F, key_encoder: K, value_encoder: V) -> Self {
        let fields = Fields::new((
            FieldEncoder::new(F1, key_encoder),
            MessageFieldEncoder::new(F2, value_encoder),
        ));
        let message = MessageEncoder::new(fields);
        let inner = Repeated::new(MessageFieldEncoder::new(field_num, message));
        MapMessageFieldEncoder { inner }
    }
}
impl<F, K, V, M> Encode for MapMessageFieldEncoder<F, K, V, M>
where
    F: Copy + Into<FieldNum>,
    K: SizedEncode + MapKeyEncode,
    V: SizedEncode + MessageEncode,
    M: IntoIterator<Item = (K::Item, V::Item)>,
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
impl<F, K, V, M> FieldEncode for MapMessageFieldEncoder<F, K, V, M>
where
    F: Copy + Into<FieldNum>,
    K: SizedEncode + MapKeyEncode,
    V: SizedEncode + MessageEncode,
    M: IntoIterator<Item = (K::Item, V::Item)>,
{
}
impl<F, K, V, M: IntoIterator> fmt::Debug for MapMessageFieldEncoder<F, K, V, M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapMessageFieldEncoder {{ .. }}")
    }
}
