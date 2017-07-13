use std::io::Read;

use {Tag, WireType};
use decode::{Decode, FieldDecode, FieldDecodeResult};
use fields::Singular;

macro_rules! check_tag {
    ($reader:expr, $actual:expr, $expected:expr) => {
        if $actual != $expected {
            return FieldDecodeResult::NotTarget($reader);
        }
    }
}

impl<R: Read, T: Decode<R>> FieldDecode<R> for Singular<T> {
    type Future = T::Future;
    fn field_decode(
        &self,
        reader: R,
        tag: Tag,
        wire_type: WireType,
        acc: Self::Value,
    ) -> FieldDecodeResult<Self::Future, R> {
        check_tag!(reader, tag, self.tag);
        panic!()
    }
}
