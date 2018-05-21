//! Encoders, decoders and related components for message fields.
use bytecodec::bytes::CopyableBytesDecoder;
use bytecodec::padding::PaddingDecoder;
use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, Result, SizedEncode};
pub use fields::Fields;
pub use oneof::Oneof;
pub use repeated_field::{MapFieldDecoder, MapFieldEncoder, MapMessageFieldDecoder,
                         MapMessageFieldEncoder, PackedFieldDecoder, PackedFieldEncoder, Repeated};

pub mod num {
    //! Field number.

    pub use field_num::*;
}
pub mod branch {
    //! Values for `Oneof` fields.

    pub use oneof::{Branch2, Branch3, Branch4, Branch5, Branch6, Branch7, Branch8};
}
pub mod value {
    //! Traits for representing encoders and decoders of field values.

    pub use value::*;
}

use field_num::FieldNum;
use message::{EmbeddedMessageDecoder, EmbeddedMessageEncoder, MessageDecode, MessageEncode};
use value::{ValueDecode, ValueEncode};
use wire::{LengthDelimitedDecoder, Tag, TagEncoder, VarintDecoder, WireType};

/// This trait allows for decoding message fields.
pub trait FieldDecode: Decode {
    /// Tries to start decoding a field.
    ///
    /// If `tag` is not a target of the decoder, `Ok(false)` will be returned.
    fn start_decoding(&mut self, tag: Tag) -> Result<bool>;
}

/// This trait allows for decoding required fields.
pub trait RequiredFieldDecode: FieldDecode {
    /// Returns `true` if this field has been present in the target input stream, otherwise `false`.
    ///
    /// Operationally, it means that the `start_decoding` method has been accepted by the decoder but
    /// the corresponding `finish_decoding` method has not been called yet.
    fn is_present(&self) -> bool;
}

/// This trait allows for encoding message fields.
pub trait FieldEncode: Encode {}

/// This trait allows for encoding required fields.
pub trait RequiredFieldEncode: FieldEncode {}

/// Encoder for required embedded message fields.
#[derive(Debug, Default)]
pub struct MessageFieldDecoder<F, D: MessageDecode> {
    inner: FieldDecoder<F, EmbeddedMessageDecoder<D>>,
}
impl<F, D: MessageDecode> MessageFieldDecoder<F, D> {
    /// Makes a new `MessageFieldDecoder` instance.
    pub fn new(field_num: F, message_decoder: D) -> Self {
        MessageFieldDecoder {
            inner: FieldDecoder::new(field_num, EmbeddedMessageDecoder::new(message_decoder)),
        }
    }
}
impl<F, D> Decode for MessageFieldDecoder<F, D>
where
    F: Copy + Into<FieldNum>,
    D: MessageDecode,
{
    type Item = D::Item;

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
impl<F, D> FieldDecode for MessageFieldDecoder<F, D>
where
    F: Copy + Into<FieldNum>,
    D: MessageDecode,
{
    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        track!(self.inner.start_decoding(tag))
    }
}
impl<F, D> RequiredFieldDecode for MessageFieldDecoder<F, D>
where
    F: Copy + Into<FieldNum>,
    D: MessageDecode,
{
    fn is_present(&self) -> bool {
        self.inner.is_present()
    }
}

/// Decoder for required scalar fields.
#[derive(Debug, Default)]
pub struct FieldDecoder<F, D: ValueDecode> {
    num: F,
    value: D,
    present: bool,
}
impl<F, D: ValueDecode> FieldDecoder<F, D> {
    /// Makes a new `FieldDecoder` instance.
    pub fn new(field_num: F, value_decoder: D) -> Self {
        FieldDecoder {
            num: field_num,
            value: value_decoder,
            present: false,
        }
    }
}
impl<F, D> Decode for FieldDecoder<F, D>
where
    F: Copy + Into<FieldNum>,
    D: ValueDecode,
{
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.value.decode(buf, eos); self.num.into())
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(
            self.present,
            ErrorKind::InvalidInput,
            "Missing required field: {:?}",
            self.num.into()
        );
        let item = track!(self.value.finish_decoding(); self.num.into())?;
        self.present = false;
        Ok(item)
    }

    fn is_idle(&self) -> bool {
        self.value.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.value.requiring_bytes()
    }
}
impl<F, D> FieldDecode for FieldDecoder<F, D>
where
    F: Copy + Into<FieldNum>,
    D: ValueDecode,
{
    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        if self.num.into() == tag.field_num {
            track_assert_eq!(self.value.wire_type(), tag.wire_type, ErrorKind::InvalidInput; tag);
            self.present = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
impl<F, D> RequiredFieldDecode for FieldDecoder<F, D>
where
    F: Copy + Into<FieldNum>,
    D: ValueDecode,
{
    fn is_present(&self) -> bool {
        self.present
    }
}

/// Decoder and encoder for optinal fields.
#[derive(Debug, Default)]
pub struct Optional<T>(T);
impl<T> Optional<T> {
    /// Makes a new `Optional` instance.
    pub fn new(inner: T) -> Self {
        Optional(inner)
    }

    /// Returns a reference to the inner field encoder/decoder.
    pub fn inner_ref(&self) -> &T {
        &self.0
    }

    /// Returns a mutable reference to the inner field encoder/decoder.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Takes the ownership of the instance, and returns the inner field encoder/decoder.
    pub fn into_inner(self) -> T {
        self.0
    }
}
impl<D: RequiredFieldDecode> Decode for Optional<D> {
    type Item = Option<D::Item>;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.0.decode(buf, eos))
    }

    fn is_idle(&self) -> bool {
        !self.0.is_present() || self.0.is_idle()
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        if self.0.is_present() {
            track!(self.0.finish_decoding()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_idle() {
            ByteCount::Finite(0)
        } else {
            self.0.requiring_bytes()
        }
    }
}
impl<D: RequiredFieldDecode> FieldDecode for Optional<D> {
    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        track!(self.0.start_decoding(tag))
    }
}
impl<E: RequiredFieldEncode> Encode for Optional<E> {
    type Item = Option<E::Item>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        if let Some(item) = item {
            track!(self.0.start_encoding(item))?;
        }
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<E: RequiredFieldEncode + SizedEncode> SizedEncode for Optional<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}
impl<E: RequiredFieldEncode> FieldEncode for Optional<E> {}

/// Decoder and encoder for optional fields which have the default values.
///
/// If a field is missing in a target input stream, the default value is used as the field value instead.
#[derive(Debug, Default)]
pub struct MaybeDefault<T>(Optional<T>);
impl<T> MaybeDefault<T> {
    /// Makes a new `MaybeDefault` instance.
    pub fn new(inner: T) -> Self {
        MaybeDefault(Optional::new(inner))
    }

    /// Returns a reference to the inner field encoder/decoder.
    pub fn inner_ref(&self) -> &T {
        self.0.inner_ref()
    }

    /// Returns a mutable reference to the inner field encoder/decoder.
    pub fn inner_mut(&mut self) -> &mut T {
        self.0.inner_mut()
    }

    /// Takes the ownership of the instance, and returns the inner field encoder/decoder.
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}
impl<D> Decode for MaybeDefault<D>
where
    D: RequiredFieldDecode,
    D::Item: Default,
{
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.0.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        if let Some(item) = track!(self.0.finish_decoding())? {
            Ok(item)
        } else {
            Ok(Default::default())
        }
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<D> FieldDecode for MaybeDefault<D>
where
    D: RequiredFieldDecode,
    D::Item: Default,
{
    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        track!(self.0.start_decoding(tag))
    }
}
impl<E> Encode for MaybeDefault<E>
where
    E: RequiredFieldEncode,
    E::Item: Default + PartialEq,
{
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        if item != E::Item::default() {
            track!(self.0.start_encoding(Some(item)))?
        }
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<E> SizedEncode for MaybeDefault<E>
where
    E: RequiredFieldEncode + SizedEncode,
    E::Item: Default + PartialEq,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}
impl<E> FieldEncode for MaybeDefault<E>
where
    E: RequiredFieldEncode,
    E::Item: Default + PartialEq,
{
}

/// Decoder for unknown fields.
///
/// This accepts any tags but the decoded values will be discarded.
#[derive(Debug)]
pub struct UnknownFieldDecoder(UnknownFieldDecoderInner);
impl UnknownFieldDecoder {
    /// Makes a new `UnknownFieldDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Decode for UnknownFieldDecoder {
    type Item = ();

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        match self.0 {
            UnknownFieldDecoderInner::None => Ok(0),
            UnknownFieldDecoderInner::Varint(ref mut d) => track!(d.decode(buf, eos)),
            UnknownFieldDecoderInner::Bit32(ref mut d) => track!(d.decode(buf, eos)),
            UnknownFieldDecoderInner::Bit64(ref mut d) => track!(d.decode(buf, eos)),
            UnknownFieldDecoderInner::LengthDelimited(ref mut d) => track!(d.decode(buf, eos)),
        }
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        match self.0 {
            UnknownFieldDecoderInner::None => {}
            UnknownFieldDecoderInner::Varint(ref mut d) => {
                track!(d.finish_decoding())?;
            }
            UnknownFieldDecoderInner::Bit32(ref mut d) => {
                track!(d.finish_decoding())?;
            }
            UnknownFieldDecoderInner::Bit64(ref mut d) => {
                track!(d.finish_decoding())?;
            }
            UnknownFieldDecoderInner::LengthDelimited(ref mut d) => {
                track!(d.finish_decoding())?;
            }
        }
        self.0 = UnknownFieldDecoderInner::None;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        match self.0 {
            UnknownFieldDecoderInner::None => true,
            UnknownFieldDecoderInner::Varint(ref d) => d.is_idle(),
            UnknownFieldDecoderInner::Bit32(ref d) => d.is_idle(),
            UnknownFieldDecoderInner::Bit64(ref d) => d.is_idle(),
            UnknownFieldDecoderInner::LengthDelimited(ref d) => d.is_idle(),
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        match self.0 {
            UnknownFieldDecoderInner::None => ByteCount::Finite(0),
            UnknownFieldDecoderInner::Varint(ref d) => d.requiring_bytes(),
            UnknownFieldDecoderInner::Bit32(ref d) => d.requiring_bytes(),
            UnknownFieldDecoderInner::Bit64(ref d) => d.requiring_bytes(),
            UnknownFieldDecoderInner::LengthDelimited(ref d) => d.requiring_bytes(),
        }
    }
}
impl FieldDecode for UnknownFieldDecoder {
    fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
        self.0 = match tag.wire_type {
            WireType::Varint => UnknownFieldDecoderInner::Varint(Default::default()),
            WireType::Bit32 => UnknownFieldDecoderInner::Bit32(Default::default()),
            WireType::Bit64 => UnknownFieldDecoderInner::Bit64(Default::default()),
            WireType::LengthDelimited => {
                UnknownFieldDecoderInner::LengthDelimited(Default::default())
            }
        };
        Ok(true)
    }
}
impl Default for UnknownFieldDecoder {
    fn default() -> Self {
        UnknownFieldDecoder(UnknownFieldDecoderInner::None)
    }
}

#[derive(Debug)]
enum UnknownFieldDecoderInner {
    None,
    Varint(VarintDecoder),
    Bit32(CopyableBytesDecoder<[u8; 4]>),
    Bit64(CopyableBytesDecoder<[u8; 8]>),
    LengthDelimited(LengthDelimitedDecoder<PaddingDecoder>),
}

/// Encoder for required embedded message fields.
#[derive(Debug, Default)]
pub struct MessageFieldEncoder<F, E> {
    inner: FieldEncoder<F, EmbeddedMessageEncoder<E>>,
}
impl<F, E> MessageFieldEncoder<F, E>
where
    E: MessageEncode + SizedEncode,
{
    /// Makes a new `MessageFieldEncoder` instance.
    pub fn new(field_num: F, message_encoder: E) -> Self {
        MessageFieldEncoder {
            inner: FieldEncoder::new(field_num, EmbeddedMessageEncoder::new(message_encoder)),
        }
    }
}
impl<F, E> Encode for MessageFieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: MessageEncode + SizedEncode,
{
    type Item = E::Item;

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
impl<F, E> SizedEncode for MessageFieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: MessageEncode + SizedEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
    }
}
impl<F, E> FieldEncode for MessageFieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: MessageEncode + SizedEncode,
{
}
impl<F, E> RequiredFieldEncode for MessageFieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: MessageEncode + SizedEncode,
{
}

/// Encoder for required scalar fields.
#[derive(Debug, Default)]
pub struct FieldEncoder<F, E> {
    num: F,
    tag: TagEncoder,
    value: E,
}
impl<F, E> FieldEncoder<F, E>
where
    E: ValueEncode,
{
    /// Makes a new `FieldEncoder` instance.
    pub fn new(field_num: F, value_encoder: E) -> Self {
        FieldEncoder {
            num: field_num,
            tag: TagEncoder::new(),
            value: value_encoder,
        }
    }
}
impl<F, E> Encode for FieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: ValueEncode,
{
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.tag, offset, buf, eos);
        bytecodec_try_encode!(self.value, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let tag = Tag::from((self.num.into(), self.value.wire_type()));
        track!(self.tag.start_encoding(tag))?;
        track!(self.value.start_encoding(item))?;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.value.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.tag
            .requiring_bytes()
            .add_for_encoding(self.value.requiring_bytes())
    }
}
impl<F, E> SizedEncode for FieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: SizedEncode + ValueEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.tag.exact_requiring_bytes() + self.value.exact_requiring_bytes()
    }
}
impl<F, E> FieldEncode for FieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: ValueEncode,
{
}
impl<F, E> RequiredFieldEncode for FieldEncoder<F, E>
where
    F: Copy + Into<FieldNum>,
    E: ValueEncode,
{
}

#[cfg(test)]
mod test {
    use bytecodec::EncodeExt;
    use bytecodec::io::IoEncodeExt;

    use super::*;
    use scalar::Fixed32Encoder;

    macro_rules! assert_encode {
        ($encoder:ty, $value:expr, $bytes:expr) => {
            let mut buf = Vec::new();
            let mut encoder: $encoder = track_try_unwrap!(EncodeExt::with_item($value));
            track_try_unwrap!(encoder.encode_all(&mut buf));
            assert_eq!(buf, $bytes);
        };
    }

    #[test]
    fn field_encoder_works() {
        assert_encode!(FieldEncoder<num::F1, Fixed32Encoder>, 123, [0x0d, 0x7b, 0x00, 0x00, 0x00]);
    }
}
