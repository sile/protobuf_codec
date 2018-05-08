use std::fmt;
use std::iter;
use std::mem;
use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, ExactBytesEncode, Result};
use bytecodec::combinator::Collect;

use field::{FieldDecode, FieldDecoder, FieldEncode, FieldEncoder, FieldsDecoder, FieldsEncoder};
use message::{EmbeddedMessageDecoder, EmbeddedMessageEncoder, MessageDecoder, MessageEncoder};
use scalar::{MapKeyDecode, MapKeyEncode, NumericDecode, NumericEncode};
use tag::{Tag, Tag1, Tag2};
use wire::{LengthDelimitedDecoder, TagAndTypeEncoder, WireDecode, WireEncode, WireType};

pub type RepeatedMessageFieldDecoder<T, V, D> =
    RepeatedFieldDecoder<T, V, EmbeddedMessageDecoder<D>>;

#[derive(Debug, Default)]
pub struct RepeatedFieldDecoder<T, V, D> {
    tag: T,
    decoder: D,
    values: V,
    is_decoding: bool,
}
impl<T, V, D> FieldDecode for RepeatedFieldDecoder<T, V, D>
where
    T: Copy + Into<Tag>,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
    D: WireDecode,
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

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_decoding {
            self.decoder.requiring_bytes()
        } else {
            ByteCount::Finite(0)
        }
    }

    fn merge(&self, mut old: Self::Item, new: Self::Item) -> Self::Item {
        old.extend(new.into_iter());
        old
    }
}

// ordinaly, it is recommended to use `RepeatedNumericFieldDecoder` instead of this
#[derive(Debug, Default)]
pub struct PackedRepeatedFieldDecoder<T, V, D>
where
    V: Default + Extend<D::Item>,
    D: NumericDecode,
{
    tag: T,
    decoder: LengthDelimitedDecoder<Collect<D, V>>,
    values: V,
    is_decoding: bool,
}
impl<T, V, D> FieldDecode for PackedRepeatedFieldDecoder<T, V, D>
where
    T: Copy + Into<Tag>,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
    D: NumericDecode,
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
            // TODO: optimize (add Collect::set_item)
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

    fn merge(&self, mut old: Self::Item, new: Self::Item) -> Self::Item {
        old.extend(new.into_iter());
        old
    }
}

#[derive(Debug, Default)]
pub struct RepeatedNumericFieldDecoder<T, V, D>
where
    V: Default + Extend<D::Item>,
    D: NumericDecode,
{
    decoder: D,
    packed_decoder: PackedRepeatedFieldDecoder<T, V, D>,
    is_decoding: bool, // TODO: remove
}
impl<T, V, D> FieldDecode for RepeatedNumericFieldDecoder<T, V, D>
where
    T: Copy + Into<Tag>,
    V: Default + Extend<D::Item> + IntoIterator<Item = D::Item>,
    D: NumericDecode,
{
    type Item = V;

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
        if self.packed_decoder.tag.into() != tag {
            Ok(false)
        } else if wire_type == WireType::LengthDelimited {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            track!(self.packed_decoder.start_decoding(tag, wire_type))?;
            self.is_decoding = true;
            Ok(true)
        } else {
            track_assert!(!self.is_decoding, ErrorKind::Other);
            self.is_decoding = true;
            Ok(true)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        if self.packed_decoder.is_decoding {
            let size = track!(self.packed_decoder.field_decode(buf, eos))?;
            if !self.packed_decoder.is_decoding {
                self.is_decoding = false;
            }
            Ok(size)
        } else {
            track_assert!(self.is_decoding, ErrorKind::Other);
            let (size, item) =
                track!(self.decoder.decode(buf, eos); self.packed_decoder.tag.into())?;
            if let Some(value) = item {
                self.packed_decoder.values.extend(iter::once(value));
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
        track!(self.packed_decoder.finish_decoding())
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_decoding {
            self.decoder.requiring_bytes()
        } else {
            ByteCount::Finite(0)
        }
    }

    fn merge(&self, mut old: Self::Item, new: Self::Item) -> Self::Item {
        old.extend(new.into_iter());
        old
    }
}

#[derive(Default)]
pub struct MapFieldDecoder<T, M, K, V>
where
    K: MapKeyDecode,
    V: WireDecode,
{
    inner: RepeatedMessageFieldDecoder<
        T,
        M,
        MessageDecoder<FieldsDecoder<(FieldDecoder<Tag1, K>, FieldDecoder<Tag2, V>)>>,
    >,
}
impl<T, M, K, V> FieldDecode for MapFieldDecoder<T, M, K, V>
where
    T: Copy + Into<Tag>,
    M: Default + Extend<(K::Value, V::Value)> + IntoIterator<Item = (K::Value, V::Value)>,
    K: MapKeyDecode,
    V: WireDecode,
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

    fn merge(&self, old: Self::Item, new: Self::Item) -> Self::Item {
        self.inner.merge(old, new)
    }
}
impl<T, M, K, V> fmt::Debug for MapFieldDecoder<T, M, K, V>
where
    K: MapKeyDecode,
    V: WireDecode,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapFieldDecoder {{ .. }}")
    }
}

#[derive(Debug)]
pub struct RepeatedFieldEncoder<T, F: IntoIterator, E> {
    inner: FieldEncoder<T, E>,
    values: Option<F::IntoIter>,
}
impl<T: Default, F: IntoIterator, E: Default> Default for RepeatedFieldEncoder<T, F, E> {
    fn default() -> Self {
        RepeatedFieldEncoder {
            inner: FieldEncoder::default(),
            values: None,
        }
    }
}
impl<T, F, E> Encode for RepeatedFieldEncoder<T, F, E>
where
    T: Copy + Into<Tag>,
    F: IntoIterator<Item = E::Item>,
    E: WireEncode,
{
    type Item = F;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        while offset < buf.len() {
            if self.inner.is_idle() {
                if let Some(item) = self.values.as_mut().and_then(|x| x.next()) {
                    track!(self.inner.force_start_encoding(item))?;
                } else {
                    self.values = None;
                    break;
                }
            }
            bytecodec_try_encode!(self.inner, offset, buf, eos);
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
impl<T, F, E> FieldEncode for RepeatedFieldEncoder<T, F, E>
where
    T: Copy + Into<Tag>,
    F: IntoIterator<Item = E::Item>,
    E: WireEncode,
{
}

#[derive(Debug)]
pub struct PackedRepeatedFieldEncoder<T, F: IntoIterator, E> {
    tag: T,
    values: Option<F::IntoIter>,
    tag_and_type_encoder: TagAndTypeEncoder,
    value_encoder: E,
}
impl<T: Default, F: IntoIterator, E: Default> Default for PackedRepeatedFieldEncoder<T, F, E> {
    fn default() -> Self {
        PackedRepeatedFieldEncoder {
            tag: T::default(),
            values: None,
            tag_and_type_encoder: TagAndTypeEncoder::new(),
            value_encoder: E::default(),
        }
    }
}
impl<T, F, E> Encode for PackedRepeatedFieldEncoder<T, F, E>
where
    T: Copy + Into<Tag>,
    F: IntoIterator<Item = E::Item>,
    E: NumericEncode,
{
    type Item = F;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.tag_and_type_encoder, offset, buf, eos);

        while offset < buf.len() {
            if self.value_encoder.is_idle() {
                if let Some(item) = self.values.as_mut().and_then(|x| x.next()) {
                    track!(self.value_encoder.start_encoding(item))?;
                } else {
                    self.values = None;
                    break;
                }
            }
            bytecodec_try_encode!(self.value_encoder, offset, buf, eos);
        }
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        let tag_and_type = (self.tag.into(), self.value_encoder.wire_type());
        track!(self.tag_and_type_encoder.start_encoding(tag_and_type))?;
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
impl<T, F, E> FieldEncode for PackedRepeatedFieldEncoder<T, F, E>
where
    T: Copy + Into<Tag>,
    F: IntoIterator<Item = E::Item>,
    E: NumericEncode,
{
}

// pub fn test<K, V>()
// where
//     K: Default + ExactBytesEncode + MapKeyEncode,
//     V: Default + ExactBytesEncode + WireEncode,
// {
//     let x: FieldsEncoder<(FieldEncoder<Tag1, K>, FieldEncoder<Tag2, V>)> = Default::default();
//     x.is_idle();

//     let x: MessageEncoder<FieldsEncoder<(FieldEncoder<Tag1, K>, FieldEncoder<Tag2, V>)>> =
//         Default::default();
//     x.is_idle();
// }

#[derive(Default)]
pub struct MapFieldEncoder<T, M: IntoIterator, K, V> {
    inner: RepeatedFieldEncoder<
        T,
        M,
        EmbeddedMessageEncoder<
            MessageEncoder<FieldsEncoder<(FieldEncoder<Tag1, K>, FieldEncoder<Tag2, V>)>>,
        >,
    >,
}
impl<T, M, K, V> Encode for MapFieldEncoder<T, M, K, V>
where
    T: Copy + Into<Tag>,
    M: IntoIterator<Item = (K::Value, V::Value)>,
    K: ExactBytesEncode + MapKeyEncode,
    V: ExactBytesEncode + WireEncode,
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
    M: IntoIterator<Item = (K::Value, V::Value)>,
    K: ExactBytesEncode + MapKeyEncode,
    V: ExactBytesEncode + WireEncode,
{
}
impl<T, M: IntoIterator, K, V> fmt::Debug for MapFieldEncoder<T, M, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapFieldEncoder {{ .. }}")
    }
}
