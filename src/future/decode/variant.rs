use std::io::Read;
use futures::{Future, Poll};

use Error;
use fields;
use traits::{DecodeField, SingularField};
use variants;
use wire::WireType;

#[derive(Debug)]
pub enum DecodeVariant2<R, A, B>
where
    R: Read,
    A: DecodeField<R>,
    B: DecodeField<R>,
{
    A(A::Future),
    B(B::Future),
}
impl<R, A, B> Future for DecodeVariant2<R, A, B>
where
    R: Read,
    A: DecodeField<R> + SingularField,
    B: DecodeField<R> + SingularField,
{
    type Item = (R, fields::Variant2<A, B>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            DecodeVariant2::A(ref mut f) => poll_variant!(f, variants::Variant2::A),
            DecodeVariant2::B(ref mut f) => poll_variant!(f, variants::Variant2::B),
        })
    }
}
impl<R, A, B> DecodeField<R> for fields::Variant2<A, B>
where
    R: Read,
    A: DecodeField<R> + SingularField,
    B: DecodeField<R> + SingularField,
{
    type Future = DecodeVariant2<R, A, B>;
    fn is_target(tag: u32) -> bool {
        A::is_target(tag) || B::is_target(tag)
    }
    fn decode_field(
        self,
        reader: R,
        tag: u32,
        wire_type: WireType,
    ) -> Result<Self::Future, Error<R>> {
        Ok(if A::is_target(tag) {
            DecodeVariant2::A(track!(A::default().decode_field(reader, tag, wire_type))?)
        } else {
            assert!(B::is_target(tag));
            DecodeVariant2::B(track!(B::default().decode_field(reader, tag, wire_type))?)
        })
    }
}
