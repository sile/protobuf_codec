use std::io::{Read, Take};
use std::marker::PhantomData;
use std::mem;
use futures::{self, Future, Poll, Async};
use futures::future::Either;

use Error;
use fields;
use traits::{Tag, Type, SingularField};
use util_futures::Finished;
use variants::Variant2;
use wire::WireType;
use wire::types::LengthDelimited;
use super::{Decode, DecodeField};
use super::futures::{VecPush, DecodeLengthDelimited};

macro_rules! track_assert_wire_type {
    ($reader:expr, $actual:expr, $expected:expr) => {
        if $actual != $expected {
            use trackable::error::ErrorKindExt;
            let cause = format!("Unexpected wire type: actual={:?}, expected={:?}",
                                $actual, $expected);
            let error = track!(::ErrorKind::Invalid.cause(cause)).into();
            return Err(::Error{stream: $reader, error})
        }
    }
}

impl<R, T, F> DecodeField<R> for fields::Field<T, F>
where
    R: Read,
    T: Tag,
    F: Type + Decode<R>,
{
    type Future = F::Future;
    fn is_target(tag: u32) -> bool {
        tag == T::number()
    }
    fn decode_field(
        reader: R,
        tag: u32,
        wire_type: WireType,
        _acc: Self::Value,
    ) -> Result<Self::Future, Error<R>> {
        assert_eq!(tag, T::number());
        track_assert_wire_type!(reader, wire_type, F::wire_type());
        Ok(F::decode(reader))
    }
}

impl<R, T, F> DecodeField<R> for fields::RepeatedField<T, F>
where
    R: Read,
    T: Tag,
    F: Type + Decode<R>,
{
    type Future = VecPush<R, F::Future, F::Value>;
    fn is_target(tag: u32) -> bool {
        tag == T::number()
    }
    fn decode_field(
        reader: R,
        tag: u32,
        wire_type: WireType,
        acc: Self::Value,
    ) -> Result<Self::Future, Error<R>> {
        assert_eq!(tag, T::number());
        track_assert_wire_type!(reader, wire_type, F::wire_type());
        Ok(VecPush::new(acc, F::decode(reader)))
    }
}


struct Packed<F>(PhantomData<F>);
impl<R: Read, F: Decode<Take<R>>> Decode<Take<R>> for Packed<F> {
    type Future = Either<DecodePacked<R, F>, Finished<Take<R>, Self::Value>>;
    fn decode(reader: Take<R>) -> Self::Future {
        if reader.limit() == 0 {
            Either::B(futures::finished((reader, Vec::new())))
        } else {
            let future = F::decode(reader);
            let values = Vec::new();
            Either::A(DecodePacked { future, values })
        }
    }
}
struct DecodePacked<R, F>
where
    R: Read,
    F: Decode<Take<R>>,
{
    future: F::Future,
    values: Vec<F::Value>,
}
impl<R: Read, F: Decode<Take<R>>> Future for DecodePacked<R, F> {
    type Item = (Take<R>, Vec<F::Value>);
    type Error = Error<Take<R>>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready((r, v)) = track!(self.future.poll())? {
            self.values.push(v);
            if r.limit() == 0 {
                let values = mem::replace(&mut self.values, Vec::new());
                return Ok(Async::Ready((r, values)));
            }
            self.future = F::decode(r);
        }
        Ok(Async::NotReady)
    }
}

pub struct DecodePackedRepeated<R, F>(DecodeLengthDelimited<R, Packed<F>>)
where
    R: Read,
    F: Decode<Take<R>>;
impl<R: Read, F: Decode<Take<R>>> Future for DecodePackedRepeated<R, F> {
    type Item = (R, Vec<F::Value>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        track!(self.0.poll())
    }
}
impl<R, T, F> DecodeField<R> for fields::PackedRepeatedField<T, F>
where
    R: Read,
    T: Tag,
    F: Type + Decode<Take<R>>,
{
    type Future = DecodePackedRepeated<R, F>;
    fn is_target(tag: u32) -> bool {
        tag == T::number()
    }
    fn decode_field(
        reader: R,
        tag: u32,
        wire_type: WireType,
        _acc: Self::Value,
    ) -> Result<Self::Future, Error<R>> {
        assert_eq!(tag, T::number());
        track_assert_wire_type!(reader, wire_type, WireType::LengthDelimited);
        Ok(DecodePackedRepeated(
            LengthDelimited::<Packed<F>>::decode(reader),
        ))
    }
}

#[derive(Debug)]
pub enum DecodeOneof2<R, A, B>
where
    R: Read,
    A: DecodeField<R>,
    B: DecodeField<R>,
{
    A(A::Future),
    B(B::Future),
}
impl<R, A, B> Future for DecodeOneof2<R, A, B>
where
    R: Read,
    A: DecodeField<R>,
    B: DecodeField<R>,
{
    type Item = (R, Option<Variant2<A::Value, B::Value>>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            DecodeOneof2::A(ref mut f) => track!(f.poll())?.map(|(r, v)| (r, Some(Variant2::A(v)))),
            DecodeOneof2::B(ref mut f) => track!(f.poll())?.map(|(r, v)| (r, Some(Variant2::B(v)))),
        })
    }
}
impl<R, A, B> DecodeField<R> for fields::Oneof<(A, B)>
where
    R: Read,
    A: DecodeField<R> + SingularField,
    B: DecodeField<R> + SingularField,
{
    type Future = DecodeOneof2<R, A, B>;
    fn is_target(tag: u32) -> bool {
        A::is_target(tag) || B::is_target(tag)
    }
    fn decode_field(
        reader: R,
        tag: u32,
        wire_type: WireType,
        _acc: Self::Value,
    ) -> Result<Self::Future, Error<R>> {
        if A::is_target(tag) {
            let f = track!(A::decode_field(reader, tag, wire_type, Default::default()))?;
            Ok(DecodeOneof2::A(f))
        } else {
            assert!(B::is_target(tag));
            let f = track!(B::decode_field(reader, tag, wire_type, Default::default()))?;
            Ok(DecodeOneof2::B(f))
        }
    }
}
