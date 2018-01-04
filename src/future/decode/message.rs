use std::io::{Read, Take};
use std::mem;
use futures::{Async, Future, Poll};

use {Decode, Error, ErrorKind, Message};
use future::decode::{DecodeLengthDelimited, DecodeMaybeVarint, DiscardWireValue};
use future::util::{self, Phase3};
use traits::DecodeField;
use types::Embedded;
use wire::WireType;
use wire::types::LengthDelimited;

pub struct DecodeMessage<R, T>
where
    R: Read,
    T: Message,
    T::Base: Decode<R>,
{
    future: <T::Base as Decode<R>>::Future,
}
impl<R: Read, T: Message> DecodeMessage<R, T>
where
    T::Base: Decode<R>,
{
    pub fn new(reader: R) -> Self {
        let future = <T::Base as Decode<R>>::decode(reader);
        DecodeMessage { future }
    }
}
impl<R: Read, T: Message> Future for DecodeMessage<R, T>
where
    T::Base: Decode<R>,
{
    type Item = (R, T);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, b)) = track!(self.future.poll())? {
            match track!(T::from_base(b)) {
                Err(e) => Err(Error {
                    stream: r,
                    error: e,
                }),
                Ok(v) => Ok(Async::Ready((r, v))),
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeEmbeddedMessage<R, T>
where
    R: Read,
    T: Message,
    T::Base: Decode<Take<R>>,
{
    future: DecodeLengthDelimited<R, T::Base>,
}
impl<R, T> Future for DecodeEmbeddedMessage<R, T>
where
    R: Read,
    T: Message,
    T::Base: Decode<Take<R>>,
{
    type Item = (R, Embedded<T>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(track!(self.future.poll())?.map(|(r, t)| (r, Embedded(t.0))))
    }
}
impl<R, T> Decode<R> for Embedded<T>
where
    R: Read,
    T: Message,
    T::Base: Decode<Take<R>>,
{
    type Future = DecodeEmbeddedMessage<R, T>;
    fn decode(reader: R) -> Self::Future {
        let future = LengthDelimited::decode(reader);
        DecodeEmbeddedMessage { future }
    }
}

struct DecodeTagAndWireType<R>(DecodeMaybeVarint<R>);
impl<R: Read> DecodeTagAndWireType<R> {
    pub fn new(reader: R) -> Self {
        DecodeTagAndWireType(DecodeMaybeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeTagAndWireType<R> {
    type Item = (R, Option<(u32, WireType)>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, v)) = track!(self.0.poll())? {
            if let Some(v) = v {
                let v = v.0;
                let tag = (v >> 3) as u32;
                let wire_type = match v & 0b111 {
                    0 => WireType::Varint,
                    1 => WireType::Bit64,
                    2 => WireType::LengthDelimited,
                    5 => WireType::Bit32,
                    w @ 3...4 => failed!(r, ErrorKind::Unsupported, "Unsupported wire type: {}", w),
                    w => failed!(r, ErrorKind::Invalid, "Unknown wire type: {}", w),
                };
                Ok(Async::Ready((r, Some((tag, wire_type)))))
            } else {
                Ok(Async::Ready((r, None)))
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}

macro_rules! define_and_impl_tuple_decoder {
    ($name:ident, $phase:ident, $(($param:ident, $num:tt)),*) => {
        pub struct $name<R, $($param),*>
        where
            R: Read,
            $($param: DecodeField<R>),*
        {
            phase: Phase3<DecodeTagAndWireType<R>,
                          util::$phase<$($param::Future),*>,
                          DiscardWireValue<R>>,
            values: ($($param),*,),
        }
        impl<R, $($param),*> Future for $name<R, $($param),*>
        where
            R: Read,
            $($param: DecodeField<R>),*
        {
            type Item = (R, ($($param),*,));
            type Error = Error<R>;
            fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
                while let Async::Ready(phase) = track!(self.phase.poll())? {
                    let next = match phase {
                        Phase3::A((r, None)) => {
                            let values = mem::replace(&mut self.values, Default::default());
                            return Ok(Async::Ready((r, values)));
                        }
                        Phase3::A((r, Some((tag, wire_type)))) => {
                            if false {
                                unreachable!()
                            } $(else if $param::is_target(tag) {
                                let v = mem::replace(&mut self.values.$num, Default::default());
                                let future = track!(v.decode_field(r, tag, wire_type))?;
                                Phase3::B(util::$phase::$param(future))
                            })*
                            else {
                                Phase3::C(DiscardWireValue::new(r, wire_type))
                            }
                        }
                        Phase3::B(phase) => {
                            match phase {
                                $(util::$phase::$param((r, v)) => {
                                    self.values.$num = v;
                                    Phase3::A(DecodeTagAndWireType::new(r))
                                })*
                            }
                        }
                        Phase3::C(r) => Phase3::A(DecodeTagAndWireType::new(r)),
                    };
                    self.phase = next;
                }
                Ok(Async::NotReady)
            }
        }
        impl<R, $($param),*> Decode<R> for ($($param),*,)
        where
            R: Read,
            $($param: DecodeField<R>),*
        {
            type Future = $name<R, $($param),*>;
            fn decode(reader: R) -> Self::Future {
                let phase = Phase3::A(DecodeTagAndWireType::new(reader));
                let values = Default::default();
                $name { phase, values }
            }
        }
    }
}

define_and_impl_tuple_decoder!(DecodeTupleMessage1, Phase1, (A, 0));
define_and_impl_tuple_decoder!(DecodeTupleMessage2, Phase2, (A, 0), (B, 1));
define_and_impl_tuple_decoder!(DecodeTupleMessage3, Phase3, (A, 0), (B, 1), (C, 2));
define_and_impl_tuple_decoder!(DecodeTupleMessage4, Phase4, (A, 0), (B, 1), (C, 2), (D, 3));
define_and_impl_tuple_decoder!(
    DecodeTupleMessage5,
    Phase5,
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4)
);
define_and_impl_tuple_decoder!(
    DecodeTupleMessage6,
    Phase6,
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5)
);
define_and_impl_tuple_decoder!(
    DecodeTupleMessage7,
    Phase7,
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6)
);
define_and_impl_tuple_decoder!(
    DecodeTupleMessage8,
    Phase8,
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7)
);
