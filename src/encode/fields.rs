use std::io::Write;
use futures::{self, Future, Poll, Async};
use futures::future::{Finished, Either};

use {Tag, WireType, Type};
use decode::futures::{Phase2, Phase3}; // TODO
use fields;
use variants::Variant2;
use wires::Varint;
use super::{Encode, EncodeError};
use super::futures::{EncodeTagAndWireType, EncodeVariant2};

#[derive(Debug)]
pub struct EncodeField<W, T>
where
    W: Write,
    T: Encode<W>,
{
    value: Option<T::Value>, // TODO
    phase: Phase2<EncodeTagAndWireType<W>, T::Future>,
}
impl<W, T> EncodeField<W, T>
where
    W: Write,
    T: Encode<W>,
{
    pub fn new(tag: Tag, wire_type: WireType, value: T::Value, writer: W) -> Self {
        let phase = Phase2::A(EncodeTagAndWireType::new(tag, wire_type, writer));
        EncodeField {
            value: Some(value),
            phase,
        }
    }
}
impl<W, T> Future for EncodeField<W, T>
where
    W: Write,
    T: Encode<W>,
{
    type Item = W;
    type Error = EncodeError<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let next = match phase {
                Phase2::A(w) => {
                    let value = self.value.take().expect("Never fails");
                    Phase2::B(T::encode(value, w))
                }
                Phase2::B(w) => return Ok(Async::Ready(w)),
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}
impl<W: Write, T> Encode<W> for fields::Singular<T>
where
    T: Type + Encode<W>,
{
    type Value = (Tag, <T as Encode<W>>::Value);
    type Future = EncodeField<W, T>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        EncodeField::new(value.0, T::wire_type(), value.1, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let key_size = <Varint as Encode<W>>::encoded_size(&(((value.0).0 << 3) as u64));
        let value_size = T::encoded_size(&value.1);
        key_size + value_size
    }
}
impl<W: Write> Encode<W> for fields::ReservedTag {
    type Value = ();
    type Future = Finished<W, EncodeError<W>>;
    fn encode(_value: Self::Value, writer: W) -> Self::Future {
        futures::finished(writer)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        0
    }
}
impl<W: Write> Encode<W> for fields::ReservedName {
    type Value = ();
    type Future = Finished<W, EncodeError<W>>;
    fn encode(_value: Self::Value, writer: W) -> Self::Future {
        futures::finished(writer)
    }
    fn encoded_size(_value: &Self::Value) -> u64 {
        0
    }
}

pub struct EncodeRepeated<W, T>
where
    W: Write,
    T: Encode<W>,
{
    tag: Tag,
    wire_type: WireType,
    values: Vec<T::Value>,
    future: Either<EncodeField<W, T>, Finished<W, EncodeError<W>>>,
}
impl<W: Write, T: Encode<W>> EncodeRepeated<W, T> {
    fn new(tag: Tag, wire_type: WireType, mut values: Vec<T::Value>, writer: W) -> Self {
        values.reverse();
        let future = if let Some(v) = values.pop() {
            Either::A(EncodeField::new(tag, wire_type, v, writer))
        } else {
            Either::B(futures::finished(writer))
        };
        EncodeRepeated {
            tag,
            wire_type,
            values,
            future,
        }
    }
}
impl<W: Write, T: Encode<W>> Future for EncodeRepeated<W, T> {
    type Item = W;
    type Error = EncodeError<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(w) = track!(self.future.poll())? {
            if let Some(v) = self.values.pop() {
                let future = Either::A(EncodeField::new(self.tag, self.wire_type, v, w));
                self.future = future;
            } else {
                return Ok(Async::Ready(w));
            }
        }
        Ok(Async::NotReady)
    }
}
impl<W: Write, T> Encode<W> for fields::Repeated<T>
where
    T: Type + Encode<W>,
{
    type Value = (Tag, Vec<<T as Encode<W>>::Value>);
    type Future = EncodeRepeated<W, T>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        EncodeRepeated::new(value.0, T::wire_type(), value.1, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let key_size = <Varint as Encode<W>>::encoded_size(&(((value.0).0 << 3) as u64));
        value.1.iter().map(|v| key_size + T::encoded_size(v)).sum()
    }
}

#[derive(Debug)]
pub struct EncodePackedRepeated<W, T>
where
    W: Write,
    T: Encode<W>,
{
    values: Vec<T::Value>,
    phase: Phase3<EncodeTagAndWireType<W>, T::Future, Finished<W, EncodeError<W>>>,
}
impl<W: Write, T: Encode<W>> EncodePackedRepeated<W, T> {
    fn new(tag: Tag, wire_type: WireType, mut values: Vec<T::Value>, writer: W) -> Self {
        values.reverse();
        let phase = if values.is_empty() {
            Phase3::C(futures::finished(writer))
        } else {
            Phase3::A(EncodeTagAndWireType::new(tag, wire_type, writer))
        };
        EncodePackedRepeated { values, phase }
    }
}
impl<W: Write, T: Encode<W>> Future for EncodePackedRepeated<W, T> {
    type Item = W;
    type Error = EncodeError<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let w = match phase {
                Phase3::A(w) => w,
                Phase3::B(w) => w,
                Phase3::C(w) => w,
            };
            if let Some(v) = self.values.pop() {
                self.phase = Phase3::B(T::encode(v, w));
            } else {
                return Ok(Async::Ready(w));
            }
        }
        Ok(Async::NotReady)
    }
}
impl<W: Write, T> Encode<W> for fields::PackedRepeated<T>
where
    T: Type + Encode<W>,
{
    type Value = (Tag, Vec<<T as Encode<W>>::Value>);
    type Future = EncodePackedRepeated<W, T>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        EncodePackedRepeated::new(value.0, T::wire_type(), value.1, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let key_size = <Varint as Encode<W>>::encoded_size(&(((value.0).0 << 3) as u64));
        key_size + value.1.iter().map(T::encoded_size).sum::<u64>()
    }
}

impl<W: Write, A, B> Encode<W> for fields::Oneof<(A, B)>
where
    A: Encode<W>,
    B: Encode<W>,
{
    type Value = Variant2<A::Value, B::Value>;
    type Future = EncodeVariant2<W, A, B>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        EncodeVariant2::new(value, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        match *value {
            Variant2::A(ref v) => A::encoded_size(v),
            Variant2::B(ref v) => B::encoded_size(v),
            Variant2::None => 0,
        }
    }
}
