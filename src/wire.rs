use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, ExactBytesEncode, Result};
use bytecodec::bytes::BytesEncoder;
use field::Tag;

use value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireType {
    Varint = 0,
    Bit32 = 5,
    Bit64 = 1,
    LengthDelimited = 2,
}

#[derive(Debug, Default)]
pub struct TagAndTypeEncoder(VarintEncoder);
impl Encode for TagAndTypeEncoder {
    type Item = (Tag, WireType);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let n = (u64::from(item.0) << 3) | (item.0 as u64);
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
pub struct LengthDelimitedEncoder<E> {
    len: VarintEncoder,
    inner: E,
}
impl<E: ExactBytesEncode> Encode for LengthDelimitedEncoder<E> {
    type Item = LengthDelimited<E::Item>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = track!(self.len.encode(buf, eos))?;
        offset += track!(self.inner.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        track!(self.inner.start_encoding(item.0))?;
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

#[derive(Debug)]
pub struct LengthDelimited<T>(pub T);
impl<T> Value for LengthDelimited<T> {
    fn wire_type(&self) -> WireType {
        WireType::LengthDelimited
    }
}

#[cfg(test)]
mod test {
    use bytecodec::Encode;
    use bytecodec::io::IoEncodeExt;

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
}
