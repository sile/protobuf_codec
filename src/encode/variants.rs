use std::io::Write;
use futures::{Future, Poll, Async};

use variants::{Variant2, Variant3};
use super::{Encode, EncodeError};

#[derive(Debug)]
pub struct EncodeVariant2<W, A, B>
where
    W: Write,
    A: Encode<W>,
    B: Encode<W>,
{
    future: Variant3<A::Future, B::Future, Option<W>>,
}
impl<W, A, B> EncodeVariant2<W, A, B>
where
    W: Write,
    A: Encode<W>,
    B: Encode<W>,
{
    pub fn new(variant: Variant2<A::Value, B::Value>, writer: W) -> Self {
        let future = match variant {
            Variant2::A(v) => Variant3::A(A::encode(v, writer)),
            Variant2::B(v) => Variant3::B(B::encode(v, writer)),
            Variant2::None => Variant3::C(Some(writer)),
        };
        EncodeVariant2 { future }
    }
}
impl<W, A, B> Future for EncodeVariant2<W, A, B>
where
    W: Write,
    A: Encode<W>,
    B: Encode<W>,
{
    type Item = W;
    type Error = EncodeError<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.future {
            Variant3::A(ref mut f) => track!(f.poll()),
            Variant3::B(ref mut f) => track!(f.poll()),
            Variant3::C(ref mut w) => {
                let w = w.take().expect("Cannot poll EncodeVariant2 twice");
                Ok(Async::Ready(w))
            }
            Variant3::None => unreachable!(),
        }
    }
}
