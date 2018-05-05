use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, ExactBytesEncode, Result};
use bytecodec::bytes::BytesEncoder;
use bytecodec::combinator::{Buffered, Length};

use tag::Tag;
use value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireType {
    Varint = 0,
    Bit32 = 5,
    Bit64 = 1,
    LengthDelimited = 2,
}

#[derive(Debug, Default)]
pub struct TagAndTypeDecoder(VarintDecoder);
impl Decode for TagAndTypeDecoder {
    type Item = (Tag, WireType);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let (size, item) = track!(self.0.decode(buf, eos))?;
        if let Some(n) = item {
            let tag = n >> 3;
            track_assert!(tag <= 0xFFFF_FFFF, ErrorKind::InvalidInput; tag);

            let wire_type = match n & 0b111 {
                0 => WireType::Varint,
                5 => WireType::Bit32,
                1 => WireType::Bit64,
                2 => WireType::LengthDelimited,
                wire_type => {
                    track_panic!(ErrorKind::InvalidInput, "Unknown wire type"; wire_type, tag)
                }
            };
            Ok((size, Some((Tag(tag as u32), wire_type))))
        } else {
            Ok((size, None))
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}

#[derive(Debug, Default)]
pub struct TagAndTypeEncoder(VarintEncoder);
impl Encode for TagAndTypeEncoder {
    type Item = (Tag, WireType);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let n = (u64::from((item.0).0) << 3) | (item.1 as u64);
        track!(self.0.start_encoding(n))
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl ExactBytesEncode for TagAndTypeEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
pub struct VarintDecoder {
    value: u64,
    index: usize,
}
impl Decode for VarintDecoder {
    type Item = u64;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        for (i, b) in buf.iter().cloned().enumerate() {
            track_assert_ne!(self.index, 10, ErrorKind::InvalidInput);
            self.value |= u64::from(b & 0b0111_1111) << (7 * self.index);
            if (b & 0b1000_0000) == 0 {
                let n = self.value;
                self.index = 0;
                self.value = 0;
                return Ok((i + 1, Some(n)));
            }
            self.index += 1;
        }
        track_assert!(!eos.is_reached(), ErrorKind::UnexpectedEos);
        Ok((buf.len(), None))
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Unknown
    }
}

#[derive(Debug, Default)]
pub struct VarintEncoder(BytesEncoder<VarintBuf>);
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
impl ExactBytesEncode for VarintEncoder {
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

#[derive(Debug, Default)]
pub struct LengthDelimitedDecoder<D> {
    len: Buffered<VarintDecoder>,
    inner: Length<D>,
}
impl<D: Decode> Decode for LengthDelimitedDecoder<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let mut offset = 0;
        if !self.len.has_item() {
            offset += track!(self.len.decode(buf, eos))?.0;
            if let Some(&len) = self.len.get_item() {
                track!(self.inner.set_expected_bytes(len))?;
            } else {
                return Ok((offset, None));
            }
        }

        let (size, item) = track!(self.inner.decode(&buf[offset..], eos))?;
        offset += size;
        if let Some(item) = item {
            let _ = self.len.take_item();
            Ok((offset, Some(item)))
        } else {
            Ok((offset, None))
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.len
            .requiring_bytes()
            .add_for_decoding(self.inner.requiring_bytes())
    }
}

#[derive(Debug, Default)]
pub struct LengthDelimitedEncoder<E> {
    len: VarintEncoder,
    inner: E,
}
impl<E: ExactBytesEncode> Encode for LengthDelimitedEncoder<E> {
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
impl<E: ExactBytesEncode> ExactBytesEncode for LengthDelimitedEncoder<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.len.exact_requiring_bytes() + self.inner.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
pub struct LengthDelimited<T>(pub T);
impl<T: Default> Value for LengthDelimited<T> {
    fn wire_type(&self) -> WireType {
        WireType::LengthDelimited
    }
}

#[cfg(test)]
mod test {
    use bytecodec::Encode;
    use bytecodec::io::{IoDecodeExt, IoEncodeExt};

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
