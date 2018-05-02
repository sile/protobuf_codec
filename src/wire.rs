use bytecodec::{ByteCount, Decode, Encode, Eos, ExactBytesEncode, Result};
// use bytecodec::fixnum::U32beDecoder;
use field::Tag;

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
pub struct VarintEncoder {}
impl Encode for VarintEncoder {
    type Item = u64;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        unimplemented!()
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        unimplemented!()
    }

    fn is_idle(&self) -> bool {
        unimplemented!()
    }

    fn requiring_bytes(&self) -> ByteCount {
        unimplemented!()
    }
}
impl ExactBytesEncode for VarintEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        unimplemented!()
    }
}
