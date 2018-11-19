//! Encoders and decoders and related components for values used in the [binary wire format].
//!
//! Since this a low-level module, developers usually do not use it directly.
//!
//! [binary wire format]: https://developers.google.com/protocol-buffers/docs/encoding
use bytecodec::bytes::BytesEncoder;
use bytecodec::combinator::{Length, Peekable};
use bytecodec::{ByteCount, Decode, DecodeExt, Encode, Eos, ErrorKind, Result, SizedEncode};

use field::num::FieldNum;

/// Field tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tag {
    /// Field number.
    pub field_num: FieldNum,

    /// Wire type of the value of the field.
    pub wire_type: WireType,
}
impl From<(FieldNum, WireType)> for Tag {
    fn from((field_num, wire_type): (FieldNum, WireType)) -> Self {
        Tag {
            field_num,
            wire_type,
        }
    }
}

/// Wire type.
///
/// `protobuf_codec` does not support deprecated types (i.e., "Start group" and "End group").
///
/// See [Message Structure] for information about each types.
///
/// [Message Structure]: https://developers.google.com/protocol-buffers/docs/encoding#structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum WireType {
    Varint = 0,
    Bit32 = 5,
    Bit64 = 1,
    LengthDelimited = 2,
}

/// Decoder for tags.
#[derive(Debug, Default)]
pub struct TagDecoder(VarintDecoder);
impl TagDecoder {
    /// Makes a new `TagDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Decode for TagDecoder {
    type Item = Tag;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.0.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let n = track!(self.0.finish_decoding())?;
        let field_num = n >> 3;
        track_assert!(field_num <= 0xFFFF_FFFF, ErrorKind::InvalidInput; field_num);

        let wire_type = match n & 0b111 {
            0 => WireType::Varint,
            5 => WireType::Bit32,
            1 => WireType::Bit64,
            2 => WireType::LengthDelimited,
            wire_type => {
                track_panic!(ErrorKind::InvalidInput, "Unknown wire type"; wire_type, field_num);
            }
        };
        let field_num = track!(FieldNum::new(field_num as u32))?;
        Ok(Tag {
            field_num,
            wire_type,
        })
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }
}

/// Encoder for tags.
#[derive(Debug, Default)]
pub struct TagEncoder(VarintEncoder);
impl TagEncoder {
    /// Makes a new `TagEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Encode for TagEncoder {
    type Item = Tag;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let n = u64::from(item.field_num.as_u32() << 3) | (item.wire_type as u64);
        track!(self.0.start_encoding(n))
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl SizedEncode for TagEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}

/// Decoder for `Varint` values.
#[derive(Debug, Default)]
pub struct VarintDecoder {
    value: u64,
    index: usize,
    idle: bool,
}
impl VarintDecoder {
    /// Makes a new `VarintDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Decode for VarintDecoder {
    type Item = u64;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        if self.idle {
            return Ok(0);
        }

        for (i, b) in buf.iter().cloned().enumerate() {
            track_assert_ne!(self.index, 10, ErrorKind::InvalidInput);
            self.value |= u64::from(b & 0b0111_1111) << (7 * self.index);
            if (b & 0b1000_0000) == 0 {
                self.idle = true;
                return Ok(i + 1);
            }
            self.index += 1;
        }
        track_assert!(!eos.is_reached(), ErrorKind::UnexpectedEos);
        Ok(buf.len())
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let n = self.value;
        self.index = 0;
        self.value = 0;
        self.idle = false;
        Ok(n)
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.idle {
            ByteCount::Finite(0)
        } else {
            ByteCount::Unknown
        }
    }

    fn is_idle(&self) -> bool {
        self.idle
    }
}

/// Encoder for `Varint` values.
#[derive(Debug, Default)]
pub struct VarintEncoder(BytesEncoder<VarintBuf>);
impl VarintEncoder {
    /// Makes a new `VarintEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Encode for VarintEncoder {
    type Item = u64;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, mut n: Self::Item) -> Result<()> {
        let mut buf = VarintBuf::new();
        loop {
            let mut b = (n & 0b0111_1111) as u8;
            n >>= 7;
            if n != 0 {
                b |= 0b1000_0000;
            }
            buf.inner[buf.len] = b;
            buf.len += 1;
            if n == 0 {
                break;
            }
        }
        track!(self.0.start_encoding(buf))
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl SizedEncode for VarintEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}

#[derive(Debug)]
struct VarintBuf {
    inner: [u8; 10],
    len: usize,
}
impl VarintBuf {
    fn new() -> Self {
        VarintBuf {
            inner: [0; 10],
            len: 0,
        }
    }
}
impl AsRef<[u8]> for VarintBuf {
    fn as_ref(&self) -> &[u8] {
        &self.inner[..self.len]
    }
}

/// Decoder for `Length-delimited` values.
#[derive(Debug, Default)]
pub struct LengthDelimitedDecoder<D> {
    len: Peekable<VarintDecoder>,
    inner: Length<D>,
}
impl<D: Decode> LengthDelimitedDecoder<D> {
    /// Makes a new `LengthDelimitedDecoder` instance.
    pub fn new(inner: D) -> Self {
        LengthDelimitedDecoder {
            len: Default::default(),
            inner: inner.length(0),
        }
    }

    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        self.inner.inner_ref()
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        self.inner.inner_mut()
    }

    /// Takes ownership of the instance and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner.into_inner()
    }
}
impl<D: Decode> Decode for LengthDelimitedDecoder<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.len.is_idle() {
            bytecodec_try_decode!(self.len, offset, buf, eos);
            let len = self.len.peek().expect("Never fails");
            track!(self.inner.set_expected_bytes(*len))?;
        }
        bytecodec_try_decode!(self.inner, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let _ = track!(self.len.finish_decoding())?;
        let item = track!(self.inner.finish_decoding())?;
        Ok(item)
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.len
            .requiring_bytes()
            .add_for_decoding(self.inner.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.len.is_idle() && self.inner.is_idle()
    }
}

/// Encoder for `Length-delimited` values.
#[derive(Debug, Default)]
pub struct LengthDelimitedEncoder<E> {
    len: VarintEncoder,
    inner: E,
}
impl<E: SizedEncode> LengthDelimitedEncoder<E> {
    /// Makes a new `LengthDelimitedEncoder` instance.
    pub fn new(inner: E) -> Self {
        LengthDelimitedEncoder {
            len: Default::default(),
            inner,
        }
    }

    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Takes ownership of the instance and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.inner
    }
}
impl<E: SizedEncode> Encode for LengthDelimitedEncoder<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = track!(self.len.encode(buf, eos))?;
        offset += track!(self.inner.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        track!(self.inner.start_encoding(item))?;
        track!(self.len.start_encoding(self.inner.exact_requiring_bytes()))?;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.len.is_idle() && self.inner.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }
}
impl<E: SizedEncode> SizedEncode for LengthDelimitedEncoder<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.len.exact_requiring_bytes() + self.inner.exact_requiring_bytes()
    }
}

#[cfg(test)]
mod tests {
    use bytecodec::io::{IoDecodeExt, IoEncodeExt};
    use bytecodec::Encode;

    use super::*;

    #[test]
    fn varint_encoder_works() {
        let mut encoder = VarintEncoder::default();

        let mut buf = Vec::new();
        track_try_unwrap!(encoder.start_encoding(1));
        track_try_unwrap!(encoder.encode_all(&mut buf));
        assert_eq!(buf, [1]);

        let mut buf = Vec::new();
        track_try_unwrap!(encoder.start_encoding(300));
        track_try_unwrap!(encoder.encode_all(&mut buf));
        assert_eq!(buf, [0b1010_1100, 0b0000_0010]);
    }

    #[test]
    fn varint_decoder_works() {
        let mut decoder = VarintDecoder::default();

        let mut buf = &[0b0000_0001, 0b1010_1100, 0b0000_0010][..];
        assert_eq!(track_try_unwrap!(decoder.decode_exact(&mut buf)), 1);
        assert_eq!(track_try_unwrap!(decoder.decode_exact(&mut buf)), 300);
        assert_eq!(buf, []);
    }
}
