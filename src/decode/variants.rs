use std::io::Read;
use futures::{Future, Poll};

use variants::Variant2;
use super::{DecodeField, DecodeError};

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
    A: DecodeField<R>,
    B: DecodeField<R>,
{
    type Item = (R, Variant2<A::Value, B::Value>);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        panic!()
    }
}
