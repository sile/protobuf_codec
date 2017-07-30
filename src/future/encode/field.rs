use std::io::Write;
use futures::{self, Future, Poll, Async};
use futures::future::{Either, Finished};

use {Encode, Error};
use fields;
use tags;
use traits::{Tag, FieldType, Packable, Map};
use types::Embedded;
use future::util::{Phase2, Phase4, WithState};
use wire::WireType;
use wire::types::Varint;
use super::EncodeVarint;

#[derive(Debug)]
pub struct EncodeField<W, T>
where
    W: Write,
    T: Encode<W>,
{
    phase: Phase2<WithState<EncodeVarint<W>, T>, T::Future>,
}
impl<W: Write, T: Encode<W>> EncodeField<W, T> {
    fn new(writer: W, tag: u32, wire_type: WireType, value: T) -> Self {
        let n = (tag << 3) as u64 | wire_type as u64;
        let phase = Phase2::A(WithState::new(Varint(n).encode(writer), value));
        EncodeField { phase }
    }
}
impl<W: Write, T: Encode<W>> Future for EncodeField<W, T> {
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let next = match phase {
                Phase2::A((w, v)) => Phase2::B(v.encode(w)),
                Phase2::B(w) => return Ok(Async::Ready(w)),
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}
impl<W, T, F> Encode<W> for fields::Field<T, F>
where
    W: Write,
    T: Tag,
    F: FieldType + Encode<W>,
{
    type Future = EncodeField<W, F>;
    fn encode(self, writer: W) -> Self::Future {
        EncodeField::new(writer, T::number(), F::wire_type(), self.value)
    }
    fn encoded_size(&self) -> u64 {
        let key_size = Encode::<W>::encoded_size(&Varint((T::number() as u64) << 3));
        let value_size = self.value.encoded_size();
        key_size + value_size
    }
}

pub struct EncodeRepeatedField<W, T>
where
    W: Write,
    T: Encode<W>,
{
    tag: u32,
    wire_type: WireType,
    values: Vec<T>,
    future: Either<EncodeField<W, T>, Finished<W, Error<W>>>,
}
impl<W: Write, T: Encode<W>> EncodeRepeatedField<W, T> {
    fn new(writer: W, tag: u32, wire_type: WireType, mut values: Vec<T>) -> Self {
        values.reverse();
        let future = if let Some(v) = values.pop() {
            Either::A(EncodeField::new(writer, tag, wire_type, v))
        } else {
            Either::B(futures::finished(writer))
        };
        EncodeRepeatedField {
            tag,
            wire_type,
            values,
            future,
        }
    }
}
impl<W: Write, T: Encode<W>> Future for EncodeRepeatedField<W, T> {
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(w) = track!(self.future.poll())? {
            if let Some(v) = self.values.pop() {
                let future = Either::A(EncodeField::new(w, self.tag, self.wire_type, v));
                self.future = future;
            } else {
                return Ok(Async::Ready(w));
            }
        }
        Ok(Async::NotReady)
    }
}
impl<W, T, F> Encode<W> for fields::RepeatedField<T, F>
where
    W: Write,
    T: Tag,
    F: FieldType + Encode<W>,
{
    type Future = EncodeRepeatedField<W, F>;
    fn encode(self, writer: W) -> Self::Future {
        EncodeRepeatedField::new(writer, T::number(), F::wire_type(), self.values)
    }
    fn encoded_size(&self) -> u64 {
        let key_size = Encode::<W>::encoded_size(&Varint((T::number() as u64) << 3));
        self.values
            .iter()
            .map(|v| key_size + v.encoded_size())
            .sum()
    }
}

#[derive(Debug)]
pub struct EncodePackedRepeatedField<W, T>
where
    W: Write,
    T: Encode<W>,
{
    values: Vec<T>,
    phase: Phase4<EncodeVarint<W>, EncodeVarint<W>, T::Future, Finished<W, Error<W>>>,
}
impl<W: Write, T: Encode<W>> EncodePackedRepeatedField<W, T> {
    fn new(writer: W, tag: u32, mut values: Vec<T>) -> Self {
        values.reverse();
        let phase = if values.is_empty() {
            Phase4::D(futures::finished(writer))
        } else {
            let n = (tag << 3) as u64 | WireType::LengthDelimited as u64;
            Phase4::A(Varint(n).encode(writer))
        };
        EncodePackedRepeatedField { values, phase }
    }
}
impl<W: Write, T: Encode<W>> Future for EncodePackedRepeatedField<W, T> {
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let w = match phase {
                Phase4::A(w) => {
                    let values_size = self.values.iter().map(T::encoded_size).sum::<u64>();
                    self.phase = Phase4::B(Varint(values_size).encode(w));
                    continue;
                }
                Phase4::B(w) => w,
                Phase4::C(w) => w,
                Phase4::D(w) => w,
            };
            if let Some(v) = self.values.pop() {
                self.phase = Phase4::C(v.encode(w));
            } else {
                return Ok(Async::Ready(w));
            }
        }
        Ok(Async::NotReady)
    }
}
impl<W, T, F> Encode<W> for fields::PackedRepeatedField<T, F>
where
    W: Write,
    T: Tag,
    F: Packable + Encode<W>,
{
    type Future = EncodePackedRepeatedField<W, F>;
    fn encode(self, writer: W) -> Self::Future {
        EncodePackedRepeatedField::new(writer, T::number(), self.values)
    }
    fn encoded_size(&self) -> u64 {
        let header_size = <Varint as Encode<W>>::encoded_size(&Varint((T::number() as u64) << 3));
        let values_size = self.values.iter().map(F::encoded_size).sum::<u64>();
        let length_size = Encode::<W>::encoded_size(&Varint(values_size));
        header_size + length_size + values_size
    }
}

pub struct EncodeMapField<W, T>
where
    W: Write,
    T: Map,
    T::Key: Encode<W>,
    T::Value: Encode<W>,
{
    tag: u32,
    pairs: T::IntoIter,
    future: Either<
        EncodeField<
            W,
            Embedded<
                (fields::Field<tags::Tag1, T::Key>,
                 fields::Field<tags::Tag2, T::Value>),
            >,
        >,
        Finished<W, Error<W>>,
    >,
}
impl<W: Write, T: Map> EncodeMapField<W, T>
where
    T::Key: Encode<W>,
    T::Value: Encode<W>,
{
    fn new(writer: W, tag: u32, map: T) -> Self {
        let mut pairs = map.into_iter();
        let future = if let Some((k, v)) = pairs.next() {
            let field = Embedded::new((k.into(), v.into()));
            Either::A(EncodeField::new(
                writer,
                tag,
                WireType::LengthDelimited,
                field,
            ))
        } else {
            Either::B(futures::finished(writer))
        };
        EncodeMapField { tag, pairs, future }
    }
}
impl<W: Write, T: Map> Future for EncodeMapField<W, T>
where
    T::Key: Encode<W>,
    T::Value: Encode<W>,
{
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(w) = track!(self.future.poll())? {
            if let Some((k, v)) = self.pairs.next() {
                let field = Embedded::new((k.into(), v.into()));
                let future = Either::A(EncodeField::new(
                    w,
                    self.tag,
                    WireType::LengthDelimited,
                    field,
                ));
                self.future = future;
            } else {
                return Ok(Async::Ready(w));
            }
        }
        Ok(Async::NotReady)
    }
}
impl<W, T, M> Encode<W> for fields::MapField<T, M>
where
    W: Write,
    T: Tag,
    M: Map,
    M::Key: Encode<W>,
    M::Value: Encode<W>,
{
    type Future = EncodeMapField<W, M>;
    fn encode(self, writer: W) -> Self::Future {
        EncodeMapField::new(writer, T::number(), self.map)
    }
    fn encoded_size(&self) -> u64 {
        let tag_size = Encode::<W>::encoded_size(&Varint((T::number() as u64) << 3));
        self.map
            .iter()
            .map(|(k, v)| {
                let entry_size = 1 + k.encoded_size() + 1 + v.encoded_size();
                let length_size = Encode::<W>::encoded_size(&Varint(entry_size));
                tag_size + length_size + entry_size
            })
            .sum()
    }
}
