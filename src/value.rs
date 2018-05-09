use bytecodec::{Decode, Encode};

use wire::WireType;

pub trait ValueDecode: Decode {
    fn wire_type(&self) -> WireType;
    fn merge_values(old: Self::Item, new: Self::Item) -> Self::Item;
}

pub trait OptionalValueDecode: ValueDecode {
    type Optional: Default;
    fn merge_optional_values(old: Self::Optional, new: Self::Optional) -> Self::Optional;
}

pub trait ValueEncode: Encode {
    fn wire_type(&self) -> WireType;
}

pub trait OptionalValueEncode: ValueEncode {
    type Optional;
    fn is_none_value(value: &Self::Optional) -> bool;
}

pub trait MapKeyDecode: ValueDecode {}

pub trait MapKeyEncode: ValueEncode {}

pub trait NumericValueDecode: ValueDecode {}

pub trait NumericValueEncode: ValueEncode {}
