use bytecodec::{ByteCount, Decode, Encode, Eos, Result};

use value::Value;
use wire::TagAndTypeEncoder;

pub type Tag = u32;

#[derive(Debug)]
pub struct Field<V> {
    pub tag: Tag,
    pub value: V,
}

#[derive(Debug, Default)]
pub struct FieldEncoder<V> {
    tag_and_type: TagAndTypeEncoder,
    v: V,
}
impl<V> Encode for FieldEncoder<V>
where
    V: Encode,
    V::Item: Value,
{
    type Item = Field<V::Item>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        unimplemented!()
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let t = (item.tag, item.value.wire_type());
        track!(self.tag_and_type.start_encoding(t))?;
        unimplemented!()
    }

    fn is_idle(&self) -> bool {
        unimplemented!()
    }

    fn requiring_bytes(&self) -> ByteCount {
        unimplemented!()
    }
}
