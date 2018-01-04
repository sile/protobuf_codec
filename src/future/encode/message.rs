use std::io::Write;
use std::mem;
use futures::{Async, Future, Poll};

use {Encode, Error, Message};
use future::util;
use traits::Field;
use types::Embedded;
use wire::types::{LengthDelimited, Varint};
use super::EncodeLengthDelimited;

pub struct EncodeMessage<W, T>
where
    W: Write,
    T: Message,
    T::Base: Encode<W>,
{
    future: <T::Base as Encode<W>>::Future,
}
impl<W: Write, T: Message> EncodeMessage<W, T>
where
    T::Base: Encode<W>,
{
    pub fn new(writer: W, message: T) -> Self {
        let future = message.into_base().encode(writer);
        EncodeMessage { future }
    }
}
impl<W: Write, T: Message> Future for EncodeMessage<W, T>
where
    T::Base: Encode<W>,
{
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        track!(self.future.poll())
    }
}

pub struct EncodeEmbeddedMessage<W, T>
where
    W: Write,
    T: Message,
    T::Base: Encode<W>,
{
    future: EncodeLengthDelimited<W, T::Base>,
}
impl<W, T> Future for EncodeEmbeddedMessage<W, T>
where
    W: Write,
    T: Message,
    T::Base: Encode<W>,
{
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        track!(self.future.poll())
    }
}
impl<W: Write, T: Message> Encode<W> for Embedded<T>
where
    T::Base: Encode<W>,
{
    type Future = EncodeEmbeddedMessage<W, T>;
    fn encode(self, writer: W) -> Self::Future {
        let future = LengthDelimited(self.0).encode(writer);
        EncodeEmbeddedMessage { future }
    }
    fn encoded_size(&self) -> u64 {
        let size = self.0.encoded_size();
        Encode::<W>::encoded_size(&Varint(size)) + size
    }
}

macro_rules! define_and_impl_tuple_encoder {
    ($name:ident, $phase:ident, [$(($prev:ident, $num:tt, $param:ident)),*], $last_param:ident) => {
        #[allow(dead_code)]
        pub struct $name<W, A, $($param),*>
        where
            W: Write,
            A: Encode<W>,
            $($param: Encode<W>),*
        {
            phase: util::$phase<A::Future, $($param::Future),*>,
            values: (A, $($param),*),
        }
        impl<W, A, $($param),*> Future for $name<W, A, $($param),*>
        where
            W: Write,
            A: Encode<W> + Field,
            $($param: Encode<W> + Field),*
        {
            type Item = W;
            type Error = Error<W>;
            #[allow(unused_variables, unreachable_code)]
            fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
                while let Async::Ready(phase) = track!(self.phase.poll())? {
                    let next = match phase {
                        $(util::$phase::$prev(w) => {
                            let v = mem::replace(&mut self.values.$num, Default::default());
                            util::$phase::$param(v.encode(w))
                        })*
                            util::$phase::$last_param(w) => return Ok(Async::Ready(w)),
                    };
                    self.phase = next;
                }
                Ok(Async::NotReady)
            }
        }
        impl<W, A, $($param),*> Encode<W> for (A, $($param),*)
        where
            W: Write,
            A: Encode<W> + Field,
            $($param: Encode<W> + Field),*
        {
            type Future = $name<W, A, $($param),*>;
            fn encode(mut self, writer: W) -> Self::Future {
                let v = mem::replace(&mut self.0, Default::default());
                let phase = util::$phase::A(v.encode(writer));
                $name {
                    phase,
                    values: self,
                }
            }
            fn encoded_size(&self) -> u64 {
                self.0.encoded_size() $(+ self.$num.encoded_size())*
            }
        }
    }
}
define_and_impl_tuple_encoder!(EncodeTupleMessage1, Phase1, [], A);
define_and_impl_tuple_encoder!(EncodeTupleMessage2, Phase2, [(A, 1, B)], B);
define_and_impl_tuple_encoder!(EncodeTupleMessage3, Phase3, [(A, 1, B), (B, 2, C)], C);
define_and_impl_tuple_encoder!(
    EncodeTupleMessage4,
    Phase4,
    [(A, 1, B), (B, 2, C), (C, 3, D)],
    D
);
define_and_impl_tuple_encoder!(
    EncodeTupleMessage5,
    Phase5,
    [(A, 1, B), (B, 2, C), (C, 3, D), (D, 4, E)],
    E
);
define_and_impl_tuple_encoder!(
    EncodeTupleMessage6,
    Phase6,
    [(A, 1, B), (B, 2, C), (C, 3, D), (D, 4, E), (E, 5, F)],
    F
);
define_and_impl_tuple_encoder!(
    EncodeTupleMessage7,
    Phase7,
    [
        (A, 1, B),
        (B, 2, C),
        (C, 3, D),
        (D, 4, E),
        (E, 5, F),
        (F, 6, G) //,
    ],
    G
);
define_and_impl_tuple_encoder!(
    EncodeTupleMessage8,
    Phase8,
    [
        (A, 1, B),
        (B, 2, C),
        (C, 3, D),
        (D, 4, E),
        (E, 5, F),
        (F, 6, G),
        (G, 7, H) //,
    ],
    H
);
