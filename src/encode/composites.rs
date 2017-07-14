use std::io::Write;
use futures::{Future, Poll, Async};

use composites;
use decode::futures::Phase2; // TODO
use wires::Varint;
use super::{Encode, EncodeError};
use super::futures::EncodeVarint;

impl<W: Write> Encode<W> for composites::Enum {
    type Value = u64;
    type Future = EncodeVarint<W>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        Varint::encode(value, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        <Varint as Encode<W>>::encoded_size(value)
    }
}

#[derive(Debug)]
pub struct EncodeMessage2<W, A, B>
where
    W: Write,
    A: Encode<W>,
    B: Encode<W>,
{
    phase: Phase2<A::Future, B::Future>,
    value_b: Option<B::Value>,
}
impl<W, A, B> EncodeMessage2<W, A, B>
where
    W: Write,
    A: Encode<W>,
    B: Encode<W>,
{
    fn new(a: A::Value, b: B::Value, writer: W) -> Self {
        let phase = Phase2::A(A::encode(a, writer));
        EncodeMessage2 {
            phase,
            value_b: Some(b),
        }
    }
}
impl<W, A, B> Future for EncodeMessage2<W, A, B>
where
    W: Write,
    A: Encode<W>,
    B: Encode<W>,
{
    type Item = W;
    type Error = EncodeError<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let next = match phase {
                Phase2::A(w) => {
                    let v = self.value_b.take().expect("Never fails");
                    Phase2::B(B::encode(v, w))
                }
                Phase2::B(w) => return Ok(Async::Ready(w)),
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}
impl<W: Write, A, B> Encode<W> for composites::Message<(A, B)>
where
    A: Encode<W>,
    B: Encode<W>,
{
    type Value = (A::Value, B::Value);
    type Future = EncodeMessage2<W, A, B>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        EncodeMessage2::new(value.0, value.1, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        A::encoded_size(&value.0) + B::encoded_size(&value.1)
    }
}

#[cfg(test)]
mod test {
    use futures::Future;

    use Tag;
    use composites::Message;
    use encode::Encode;
    use fields::Singular;
    use scalars::Int32;

    #[test]
    fn it_works() {
        type M = Message<(Singular<Int32>, Singular<Int32>)>;
        let values = ((Tag(1), 150), (Tag(2), 150));
        let bytes = track_try_unwrap!(M::encode(values, Vec::new()).wait());
        assert_eq!(bytes, [0x08, 0x96, 0x01, 0x10, 0x96, 0x01]);
    }
}
