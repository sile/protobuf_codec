use bytecodec::{Decode, Encode, ExactBytesEncode, Result};

use wire::WireType;

pub trait ValueDecode: Decode {
    fn wire_type(&self) -> WireType;
    fn merge_values(old: &mut Self::Item, new: Self::Item);
}

pub trait OptionalValueDecode: ValueDecode {
    type Optional: Default + From<Self::Item>;
    fn merge_optional_values(old: &mut Self::Optional, new: Self::Optional);
}

pub trait ValueEncode: Encode {
    fn wire_type(&self) -> WireType;
}

pub trait OptionalValueEncode: ValueEncode {
    type Optional;
    fn start_encoding_if_needed(&mut self, item: Self::Optional) -> Result<()>;
}

pub trait MapKeyDecode: ValueDecode {}

pub trait MapKeyEncode: ValueEncode {}

pub trait NumericValueDecode: ValueDecode {}

pub trait NumericValueEncode: ValueEncode + ExactBytesEncode {}
