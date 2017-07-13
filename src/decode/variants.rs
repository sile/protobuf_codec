use std::io::Read;
use futures::{Future, Poll, Async};

use variants::{Variant2, Variant3};
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
    None(Option<R>),
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
        Ok(match *self {
            DecodeVariant2::A(ref mut f) => track!(f.poll())?.map(|(r, v)| (r, Variant2::A(v))),
            DecodeVariant2::B(ref mut f) => track!(f.poll())?.map(|(r, v)| (r, Variant2::B(v))),
            DecodeVariant2::None(ref mut r) => {
                let r = r.take().expect("Cannot poll DecodeVariant2 twice");
                Async::Ready((r, Variant2::None))
            }
        })
    }
}

#[derive(Debug)]
pub enum DecodeVariant3<R, A, B, C>
where
    R: Read,
    A: DecodeField<R>,
    B: DecodeField<R>,
    C: DecodeField<R>,
{
    A(A::Future),
    B(B::Future),
    C(C::Future),
    None(Option<R>),
}
impl<R, A, B, C> Future for DecodeVariant3<R, A, B, C>
where
    R: Read,
    A: DecodeField<R>,
    B: DecodeField<R>,
    C: DecodeField<R>,
{
    type Item = (R, Variant3<A::Value, B::Value, C::Value>);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            DecodeVariant3::A(ref mut f) => track!(f.poll())?.map(|(r, v)| (r, Variant3::A(v))),
            DecodeVariant3::B(ref mut f) => track!(f.poll())?.map(|(r, v)| (r, Variant3::B(v))),
            DecodeVariant3::C(ref mut f) => track!(f.poll())?.map(|(r, v)| (r, Variant3::C(v))),
            DecodeVariant3::None(ref mut r) => {
                let r = r.take().expect("Cannot poll DecodeVariant3 twice");
                Async::Ready((r, Variant3::None))
            }
        })
    }
}
