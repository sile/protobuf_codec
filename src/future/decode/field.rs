use std::io::{Read, Take};
use std::marker::PhantomData;
use std::mem;
use futures::{self, Future, Poll, Async};
use futures::future::Either;

use {Error, Decode};
use future::decode::{DecodeInto, DecodeLengthDelimited};
use future::util::Finished;
use fields;
use tags;
use traits::{Tag, FieldType, Packable, DecodeField, Map};
use wire::WireType;
use wire::types::LengthDelimited;

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
    F: FieldType + Decode<R>,
{
    type Future = DecodeInto<R, F, Self>;
    fn is_target(tag: u32) -> bool {
        tag == T::number()
    }
    fn decode_field(
        self,
        reader: R,
        tag: u32,
        wire_type: WireType,
    ) -> Result<Self::Future, Error<R>> {
        assert_eq!(tag, T::number());
        track_assert_wire_type!(reader, wire_type, F::wire_type());
        Ok(Decode::decode_into(reader))
    }
}

impl<R, T, F> DecodeField<R> for fields::RepeatedField<T, F>
where
    R: Read,
    T: Tag,
    F: FieldType + Decode<R>,
{
    type Future = DecodeRepeatedField<R, T, F>;
    fn is_target(tag: u32) -> bool {
        tag == T::number()
    }
    fn decode_field(
        self,
        reader: R,
        tag: u32,
        wire_type: WireType,
    ) -> Result<Self::Future, Error<R>> {
        assert_eq!(tag, T::number());
        track_assert_wire_type!(reader, wire_type, F::wire_type());
        Ok(DecodeRepeatedField {
            future: F::decode(reader),
            values: self.values,
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct DecodeRepeatedField<R, T, F>
where
    R: Read,
    F: Decode<R>,
{
    future: F::Future,
    values: Vec<F>,
    _phantom: PhantomData<T>,
}
impl<R, T, F> Future for DecodeRepeatedField<R, T, F>
where
    R: Read,
    T: Tag,
    F: FieldType + Decode<R>,
{
    type Item = (R, fields::RepeatedField<T, F>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, v)) = track!(self.future.poll())? {
            self.values.push(v);
            let f = fields::RepeatedField {
                tag: T::default(),
                values: mem::replace(&mut self.values, Vec::new()),
            };
            Ok(Async::Ready((r, f)))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodePackedRepeatedField<R, T, F>
where
    R: Read,
    F: Decode<Take<R>>,
{
    future: DecodeLengthDelimited<R, Packed<F>>,
    _phantom: PhantomData<T>,
}
impl<R, T, F> Future for DecodePackedRepeatedField<R, T, F>
where
    R: Read,
    T: Tag,
    F: Packable + Decode<Take<R>>,
{
    type Item = (R, fields::PackedRepeatedField<T, F>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(track!(self.future.poll())?.map(|(r, v)| {
            let f = fields::PackedRepeatedField {
                tag: T::default(),
                values: (v.0).0,
            };
            (r, f)
        }))
    }
}
impl<R, T, F> DecodeField<R> for fields::PackedRepeatedField<T, F>
where
    R: Read,
    T: Tag,
    F: Packable + Decode<Take<R>>,
{
    type Future = DecodePackedRepeatedField<R, T, F>;
    fn is_target(tag: u32) -> bool {
        tag == T::number()
    }
    fn decode_field(
        self,
        reader: R,
        tag: u32,
        wire_type: WireType,
    ) -> Result<Self::Future, Error<R>> {
        assert_eq!(tag, T::number());
        track_assert_wire_type!(reader, wire_type, WireType::LengthDelimited);
        Ok(DecodePackedRepeatedField {
            future: LengthDelimited::decode(reader),
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug)]
struct Packed<F>(Vec<F>);
impl<R: Read, F: Decode<Take<R>>> Decode<Take<R>> for Packed<F> {
    type Future = Either<DecodePacked<R, F>, Finished<Take<R>, Self>>;
    fn decode(reader: Take<R>) -> Self::Future {
        if reader.limit() == 0 {
            Either::B(futures::finished((reader, Packed(Vec::new()))))
        } else {
            let future = F::decode(reader);
            let values = Vec::new();
            Either::A(DecodePacked { future, values })
        }
    }
}

#[derive(Debug)]
struct DecodePacked<R, F>
where
    R: Read,
    F: Decode<Take<R>>,
{
    future: F::Future,
    values: Vec<F>,
}
impl<R: Read, F: Decode<Take<R>>> Future for DecodePacked<R, F> {
    type Item = (Take<R>, Packed<F>);
    type Error = Error<Take<R>>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready((r, v)) = track!(self.future.poll())? {
            self.values.push(v);
            if r.limit() == 0 {
                let values = mem::replace(&mut self.values, Vec::new());
                return Ok(Async::Ready((r, Packed(values))));
            }
            self.future = F::decode(r);
        }
        Ok(Async::NotReady)
    }
}

pub struct DecodeMapField<R, T, M>
where
    R: Read,
    M: Map + Decode<R>,
    M::Key: Decode<Take<R>>,
    M::Value: Decode<Take<R>>,
{
    future: DecodeMapEntry<R, M>,
    map: M,
    _phantom: PhantomData<T>,
}
impl<R, T, M> Future for DecodeMapField<R, T, M>
where
    R: Read,
    T: Tag,
    M: Map + Decode<R>,
    M::Key: Decode<Take<R>>,
    M::Value: Decode<Take<R>>,
{
    type Item = (R, fields::MapField<T, M>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, k, v)) = track!(self.future.poll())? {
            self.map.insert(k, v);
            let f = fields::MapField {
                tag: T::default(),
                map: mem::replace(&mut self.map, Default::default()),
            };
            Ok(Async::Ready((r, f)))
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R, T, M> DecodeField<R> for fields::MapField<T, M>
where
    R: Read,
    T: Tag,
    M: Map + Decode<R>,
    M::Key: Decode<Take<R>>,
    M::Value: Decode<Take<R>>,
{
    type Future = DecodeMapField<R, T, M>;
    fn is_target(tag: u32) -> bool {
        tag == T::number()
    }
    fn decode_field(
        self,
        reader: R,
        tag: u32,
        wire_type: WireType,
    ) -> Result<Self::Future, Error<R>> {
        assert_eq!(tag, T::number());
        track_assert_wire_type!(reader, wire_type, WireType::LengthDelimited);
        let future = LengthDelimited::decode(reader);
        Ok(DecodeMapField {
            future: DecodeMapEntry { future },
            map: self.map,
            _phantom: PhantomData,
        })
    }
}

struct DecodeMapEntry<R, M>
where
    R: Read,
    M: Map,
    M::Key: Decode<Take<R>>,
    M::Value: Decode<Take<R>>,
{
    future: DecodeLengthDelimited<
        R,
        (fields::Field<tags::Tag1, M::Key>,
         fields::Field<tags::Tag2, M::Value>),
    >,
}
impl<R, M> Future for DecodeMapEntry<R, M>
where
    R: Read,
    M: Map,
    M::Key: Decode<Take<R>>,
    M::Value: Decode<Take<R>>,
{
    type Item = (R, M::Key, M::Value);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(track!(self.future.poll())?.map(|(r, v)| {
            (r, (v.0).0.value, (v.0).1.value)
        }))
    }
}
