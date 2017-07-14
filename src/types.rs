use std::marker::PhantomData;

use traits::{Type, Field};
use wire::WireType;

pub struct Int32;
impl Type for Int32 {
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

pub struct Message<Fields>(PhantomData<Fields>);
impl<A, B> Type for Message<(A, B)>
where
    A: Field,
    B: Field,
{
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}
