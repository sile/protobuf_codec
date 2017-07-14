use std::io::Read;
use std::mem;
use futures::{Future, Poll, Async};

use Error;
use types;
use util_futures::Phase4;
use super::{Decode, DecodeField};
use super::futures::{DecodeTagAndWireType, DiscardWireValue};

pub struct DecodeMessage2<R, A, B>
where
    R: Read,
    A: DecodeField<R>,
    B: DecodeField<R>,
{
    phase: Phase4<DecodeTagAndWireType<R>, A::Future, B::Future, DiscardWireValue<R>>,
    values: (A::Value, B::Value),
}
impl<R, A, B> Future for DecodeMessage2<R, A, B>
where
    R: Read,
    A: DecodeField<R>,
    B: DecodeField<R>,
{
    type Item = (R, (A::Value, B::Value));
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let next = match phase {
                Phase4::A((r, None)) => {
                    let values = mem::replace(&mut self.values, Default::default());
                    return Ok(Async::Ready((r, values)));
                }
                Phase4::A((r, Some((tag, wire_type)))) => {
                    if A::is_target(tag) {
                        let v = mem::replace(&mut self.values.0, Default::default());
                        Phase4::B(track!(A::decode_field(r, tag, wire_type, v))?)
                    } else if B::is_target(tag) {
                        let v = mem::replace(&mut self.values.1, Default::default());
                        Phase4::C(track!(B::decode_field(r, tag, wire_type, v))?)
                    } else {
                        Phase4::D(DiscardWireValue::new(r, wire_type))
                    }
                }
                Phase4::B((r, v)) => {
                    self.values.0 = v;
                    Phase4::A(DecodeTagAndWireType::new(r))
                }
                Phase4::C((r, v)) => {
                    self.values.1 = v;
                    Phase4::A(DecodeTagAndWireType::new(r))
                }
                Phase4::D((r, ())) => {
                    Phase4::A(DecodeTagAndWireType::new(r))
                }
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}
impl<R, A, B> Decode<R> for types::Message<(A, B)>
where
    R: Read,
    A: DecodeField<R>,
    B: DecodeField<R>,
{
    type Value = (A::Value, B::Value);
    type Future = DecodeMessage2<R, A, B>;
    fn decode(reader: R) -> Self::Future {
        let phase = Phase4::A(DecodeTagAndWireType::new(reader));
        let values = Default::default();
        DecodeMessage2 { phase, values }
    }
}
