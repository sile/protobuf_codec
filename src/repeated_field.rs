use bytecodec::combinator::Collect;
use bytecodec::{ByteCount, Decode, DecodeExt, Encode, Eos, ErrorKind, ExactBytesEncode, Result};
use std::fmt;
use std::iter;
use std::mem;

use field::{FieldDecode, FieldDecoder, FieldEncode, FieldEncoder, Fields,
            RepeatedMessageFieldDecoder, RepeatedMessageFieldEncoder};
use message::{EmbeddedMessageDecoder, EmbeddedMessageEncoder, MessageDecoder, MessageEncoder};
use scalar::BytesEncoder;
use tag::{Tag, Tag1, Tag2};
use value::{MapKeyDecode, MapKeyEncode, NumericValueDecode, NumericValueEncode, ValueDecode,
            ValueEncode};
use wire::{LengthDelimitedDecoder, TagAndTypeEncoder, WireType};

/// Decoder for repeated fields.
#[derive(Debug, Default)]
pub struct RepeatedFieldDecoder<T, V, D> {
    tag: T,
    decoder: D,
    values: V,
    is_decoding: bool,
}
impl<T, V: Default, D> RepeatedFieldDecoder<T, V, D> {
    /// Makes a new `RepeatedFieldDecoder` instance.
    pub fn new(tag: T, value_decoder: D) -> Self {
        RepeatedFieldDecoder {
            tag,
            decoder: value_decoder,
            values: V::default(),
            is_decoding: false,
        }
    }
}
impl<T, V, D> FieldDecode for RepeatedFieldDecoder<T, V, D>
where
    T: Copy + Into<Tag>,
    D: ValueDecode,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
        if self.tag.into() == tag {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track_assert_eq!(self.decoder.wire_type(), wire_type, ErrorKind::InvalidInput; tag);
            self.is_decoding = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let (size, item) = track!(self.decoder.decode(buf, eos); self.tag.into())?;
        if let Some(value) = item {
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
        self.decoder.requiring_bytes()
    }

    fn merge_fields(old: &mut Self::Item, new: Self::Item) {
        old.extend(new.into_iter());
    }
}

/// Encoder for repeated fields.
#[derive(Debug)]
pub struct RepeatedFieldEncoder<T, V: IntoIterator, E> {
    field: FieldEncoder<T, E>,
    values: Option<V::IntoIter>,
}
impl<T, V: IntoIterator, E: ValueEncode> RepeatedFieldEncoder<T, V, E> {
    /// Makes new `RepeatedFieldEncoder` instance.
    pub fn new(tag: T, value_encoder: E) -> Self {
        RepeatedFieldEncoder {
            field: FieldEncoder::new(tag, value_encoder),
            values: None,
        }
    }
}
impl<T: Default, V: IntoIterator, E: Default> Default for RepeatedFieldEncoder<T, V, E> {
    fn default() -> Self {
        RepeatedFieldEncoder {
            field: FieldEncoder::default(),
            values: None,
        }
    }
}
impl<T, E, V> Encode for RepeatedFieldEncoder<T, V, E>
where
    T: Copy + Into<Tag>,
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
impl<T, V, E> FieldEncode for RepeatedFieldEncoder<T, V, E>
where
    T: Copy + Into<Tag>,
    V: IntoIterator<Item = E::Item>,
    E: ValueEncode,
{
}

/// Decoder for packed repeated fields.
///
/// Ordinarily, it is recommended to use `RepeatedNumericFieldDecoder` instead of this.
#[derive(Debug, Default)]
pub struct PackedRepeatedFieldDecoder<T, V, D>
where
    V: Default + Extend<D::Item>,
    D: NumericValueDecode,
{
    tag: T,
    decoder: LengthDelimitedDecoder<Collect<D, V>>,
    values: V,
    is_decoding: bool,
}
impl<T, V, D> PackedRepeatedFieldDecoder<T, V, D>
where
    V: Default + Extend<D::Item>,
    D: NumericValueDecode,
{
    /// Makes a new `PackedRepeatedFieldDecoder` instance.
    pub fn new(tag: T, value_decoder: D) -> Self {
        PackedRepeatedFieldDecoder {
            tag,
            decoder: LengthDelimitedDecoder::new(value_decoder.collect()),
            values: V::default(),
            is_decoding: false,
        }
    }

    fn inner_mut(&mut self) -> &mut D {
        self.decoder.inner_mut().inner_mut()
    }
}
impl<T, V, D> FieldDecode for PackedRepeatedFieldDecoder<T, V, D>
where
    T: Copy + Into<Tag>,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
    D: NumericValueDecode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag, _: WireType) -> Result<bool> {
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

        let (size, item) = track!(self.decoder.decode(buf, eos); self.tag.into())?;
        if let Some(values) = item {
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

    fn merge_fields(old: &mut Self::Item, new: Self::Item) {
        old.extend(new.into_iter());
    }
}

/// Decoder for repeated numeric fields.
///
/// This can decode numeric fields regardless of whether they are packed or not.
#[derive(Debug, Default)]
pub struct RepeatedNumericFieldDecoder<T, V, D>
where
    V: Default + Extend<D::Item>,
    D: NumericValueDecode,
{
    decoder: PackedRepeatedFieldDecoder<T, V, D>,
    is_decoding: bool,
}
impl<T, V, D> RepeatedNumericFieldDecoder<T, V, D>
where
    V: Default + Extend<D::Item>,
    D: NumericValueDecode,
{
    /// Makes a new `RepeatedNumericFieldDecoder` instance.
    pub fn new(tag: T, value_decoder: D) -> Self {
        RepeatedNumericFieldDecoder {
            decoder: PackedRepeatedFieldDecoder::new(tag, value_decoder),
            is_decoding: false,
        }
    }
}
impl<T, V, D> FieldDecode for RepeatedNumericFieldDecoder<T, V, D>
where
    T: Copy + Into<Tag>,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
    D: NumericValueDecode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
        if self.decoder.tag.into() != tag {
            Ok(false)
        } else if wire_type == WireType::LengthDelimited {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track!(self.decoder.start_decoding(tag, wire_type))?;
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
            let (size, item) =
                track!(self.decoder.inner_mut().decode(buf, eos); self.decoder.tag.into())?;
            if let Some(value) = item {
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

    fn merge_fields(old: &mut Self::Item, new: Self::Item) {
        old.extend(new.into_iter());
    }
}

/// Decoder for map fields.
#[derive(Default)]
pub struct MapFieldDecoder<T, M, K, V>
where
    K: MapKeyDecode,
    V: ValueDecode,
{
    inner: RepeatedMessageFieldDecoder<
        T,
        M,
        MessageDecoder<Fields<(FieldDecoder<Tag1, K>, FieldDecoder<Tag2, V>)>>,
    >,
}
impl<T, M: Default, K: MapKeyDecode, V: ValueDecode> MapFieldDecoder<T, M, K, V> {
    /// Makes a new `MapFieldDecoder` instance.
    pub fn new(tag: T, key_decoder: K, value_decoder: V) -> Self {
        let fields = Fields::new((
            FieldDecoder::new(Tag1, key_decoder),
            FieldDecoder::new(Tag2, value_decoder),
        ));
        let message = MessageDecoder::new(fields);
        let inner = RepeatedMessageFieldDecoder::new(tag, EmbeddedMessageDecoder::new(message));
        MapFieldDecoder { inner }
    }
}
impl<T, M, K, V> FieldDecode for MapFieldDecoder<T, M, K, V>
where
    T: Copy + Into<Tag>,
    M: Default + Extend<(K::Item, V::Item)> + IntoIterator<Item = (K::Item, V::Item)>,
    K: MapKeyDecode,
    V: ValueDecode,
{
    type Item = M;

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
        track!(self.inner.start_decoding(tag, wire_type))
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

    fn merge_fields(old: &mut Self::Item, new: Self::Item) {
        old.extend(new.into_iter());
    }
}
impl<T, M, K, V> fmt::Debug for MapFieldDecoder<T, M, K, V>
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
pub struct PackedRepeatedFieldEncoder<T, V: IntoIterator, E> {
    tag: T,
    values: Option<V::IntoIter>,
    tag_and_type_encoder: TagAndTypeEncoder,
    value_encoder: E,
    bytes_encoder: BytesEncoder,
}
impl<T, V: IntoIterator, E> PackedRepeatedFieldEncoder<T, V, E> {
    /// Makes a new `PackedRepeatedFieldEncoder` instance.
    pub fn new(tag: T, value_encoder: E) -> Self {
        PackedRepeatedFieldEncoder {
            tag,
            values: None,
            tag_and_type_encoder: TagAndTypeEncoder::new(),
            value_encoder,
            bytes_encoder: BytesEncoder::new(),
        }
    }
}
impl<T: Default, V: IntoIterator, E: Default> Default for PackedRepeatedFieldEncoder<T, V, E> {
    fn default() -> Self {
        PackedRepeatedFieldEncoder {
            tag: T::default(),
            values: None,
            tag_and_type_encoder: TagAndTypeEncoder::new(),
            value_encoder: E::default(),
            bytes_encoder: BytesEncoder::default(),
        }
    }
}
impl<T, V, E> Encode for PackedRepeatedFieldEncoder<T, V, E>
where
    T: Copy + Into<Tag>,
    V: IntoIterator<Item = E::Item>,
    E: NumericValueEncode,
{
    type Item = V;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.tag_and_type_encoder, offset, buf, eos);
        bytecodec_try_encode!(self.bytes_encoder, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let tag_and_type = (self.tag.into(), self.bytes_encoder.wire_type());
        track!(self.tag_and_type_encoder.start_encoding(tag_and_type))?;
        let mut buf = Vec::new();
        for v in item.into_iter() {
            track!(self.value_encoder.start_encoding(v))?;

            let eos = Eos::new(false);
            let old_len = buf.len();
            let new_len = old_len + self.value_encoder.exact_requiring_bytes() as usize;
            buf.resize(new_len, 0);
            track!(self.value_encoder.encode(&mut buf[old_len..], eos))?;
        }
        track!(self.bytes_encoder.start_encoding(buf))?;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.bytes_encoder.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }
}
impl<T, F, E> ExactBytesEncode for PackedRepeatedFieldEncoder<T, F, E>
where
    T: Copy + Into<Tag>,
    F: IntoIterator<Item = E::Item>,
    E: NumericValueEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.tag_and_type_encoder.exact_requiring_bytes()
            + self.bytes_encoder.exact_requiring_bytes()
    }
}
impl<T, F, E> FieldEncode for PackedRepeatedFieldEncoder<T, F, E>
where
    T: Copy + Into<Tag>,
    F: IntoIterator<Item = E::Item>,
    E: NumericValueEncode,
{
}

/// Encoder for map fields.
#[derive(Default)]
pub struct MapFieldEncoder<T, M: IntoIterator, K, V> {
    inner: RepeatedMessageFieldEncoder<
        T,
        M,
        MessageEncoder<Fields<(FieldEncoder<Tag1, K>, FieldEncoder<Tag2, V>)>>,
    >,
}
impl<T, M, K, V> MapFieldEncoder<T, M, K, V>
where
    T: Copy + Into<Tag>,
    M: IntoIterator<Item = (K::Item, V::Item)>,
    K: ExactBytesEncode + MapKeyEncode,
    V: ExactBytesEncode + ValueEncode,
{
    /// Makes a new `MapFieldEncoder` instance.
    pub fn new(tag: T, key_encoder: K, value_encoder: V) -> Self {
        let fields = Fields::new((
            FieldEncoder::new(Tag1, key_encoder),
            FieldEncoder::new(Tag2, value_encoder),
        ));
        let message = MessageEncoder::new(fields);
        let inner = RepeatedMessageFieldEncoder::new(tag, EmbeddedMessageEncoder::new(message));
        MapFieldEncoder { inner }
    }
}
impl<T, M, K, V> Encode for MapFieldEncoder<T, M, K, V>
where
    T: Copy + Into<Tag>,
    M: IntoIterator<Item = (K::Item, V::Item)>,
    K: ExactBytesEncode + MapKeyEncode,
    V: ExactBytesEncode + ValueEncode,
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
impl<T, M, K, V> FieldEncode for MapFieldEncoder<T, M, K, V>
where
    T: Copy + Into<Tag>,
    M: IntoIterator<Item = (K::Item, V::Item)>,
    K: ExactBytesEncode + MapKeyEncode,
    V: ExactBytesEncode + ValueEncode,
{
}
impl<T, M: IntoIterator, K, V> fmt::Debug for MapFieldEncoder<T, M, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapFieldEncoder {{ .. }}")
    }
}
