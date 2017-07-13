use std::io::{Read, Take};
use std::mem;
use futures::{self, Future, Poll, Async};
use futures::future::{Either, Finished};

use {Type, Payload, Tag, WireType};
use composites::{Packed, Enum, Message};
use fields;
use variants::Variant3;
use wires::{Varint, LengthDelimited};
use super::{Decode, DecodePayload, DecodeError, DecodeField};
use super::futures::{DecodeLengthDelimited, DecodeVarint, Phase2, DecodeTagAndWireType,
                     DecodeVariant3};

impl<R: Read> Decode<R> for Enum {
    type Future = DecodeVarint<R>;
    fn decode(self, reader: R) -> Self::Future {
        Varint.decode(reader)
    }
}

pub struct DecodePacked<R, T>(DecodeLengthDelimited<R, Repeat<T>>)
where
    R: Read,
    T: Decode<Take<R>> + Clone;
impl<R, T> Future for DecodePacked<R, T>
where
    R: Read,
    T: Decode<Take<R>> + Clone,
{
    type Item = (R, Vec<T::Value>);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        track!(self.0.poll())
    }
}
impl<R, T> Decode<R> for Packed<T>
where
    R: Read,
    T: Decode<Take<R>> + Clone,
{
    type Future = DecodePacked<R, T>;
    fn decode(self, reader: R) -> Self::Future {
        DecodePacked(LengthDelimited(Repeat(self.0)).decode(reader))
    }
}

#[derive(Debug)]
struct Repeat<T>(T);
impl<T: Type> Payload for Repeat<T> {
    type Value = Vec<T::Value>;
}
impl<R: Read, T> DecodePayload<R> for Repeat<T>
where
    T: Decode<Take<R>> + Clone,
{
    type Future = Either<
        DecodeRepeat<R, T>,
        Finished<(Take<R>, Vec<T::Value>), DecodeError<Take<R>>>,
    >;
    fn decode_payload(self, reader: Take<R>) -> Self::Future {
        if reader.limit() == 0 {
            Either::B(futures::finished((reader, Vec::new())))
        } else {
            Either::A(DecodeRepeat::new(self.0, reader))
        }
    }
}

#[derive(Debug)]
struct DecodeRepeat<R, T>
where
    R: Read,
    T: Decode<Take<R>>,
{
    future: T::Future,
    values: Vec<T::Value>,
    value_type: T,
}
impl<R, T> DecodeRepeat<R, T>
where
    R: Read,
    T: Decode<Take<R>> + Clone,
{
    fn new(value_type: T, reader: Take<R>) -> Self {
        DecodeRepeat {
            future: value_type.clone().decode(reader),
            values: Vec::new(),
            value_type,
        }
    }
}
impl<R, T> Future for DecodeRepeat<R, T>
where
    R: Read,
    T: Decode<Take<R>> + Clone,
{
    type Item = (Take<R>, Vec<T::Value>);
    type Error = DecodeError<Take<R>>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready((r, v)) = track!(self.future.poll())? {
            self.values.push(v);
            if r.limit() == 0 {
                let values = mem::replace(&mut self.values, Vec::new());
                return Ok(Async::Ready((r, values)));
            }
            self.future = self.value_type.clone().decode(r);
        }
        Ok(Async::NotReady)
    }
}

pub struct DecodeMessage2<R, A, B>
where
    R: Read,
    A: DecodeField<Take<R>>,
    B: DecodeField<Take<R>>,
{
    message: Message<(A, B)>,
    value0: Option<A::Value>,
    value1: Option<B::Value>,
    phase: Phase2<DecodeTagAndWireType<Take<R>>, DecodeVariant3<Take<R>, A, B, fields::Ignore>>,
}
impl<R, A, B> DecodeMessage2<R, A, B>
where
    R: Read,
    A: DecodeField<Take<R>>,
    B: DecodeField<Take<R>>,
{
    pub fn new(message: Message<(A, B)>, reader: Take<R>) -> Self {
        let phase = if reader.limit() > 0 {
            Phase2::A(DecodeTagAndWireType::new(reader))
        } else {
            Phase2::B(DecodeVariant3::None(Some(reader)))
        };
        DecodeMessage2 {
            message,
            value0: Some(Default::default()),
            value1: Some(Default::default()),
            phase,
        }
    }
    fn decode_field(
        &self,
        reader: Take<R>,
        tag: Tag,
        wire_type: WireType,
    ) -> DecodeVariant3<Take<R>, A, B, fields::Ignore> {
        panic!()
    }
}
impl<R, A, B> Future for DecodeMessage2<R, A, B>
where
    R: Read,
    A: DecodeField<Take<R>>,
    B: DecodeField<Take<R>>,
{
    type Item = (Take<R>, (A::Value, B::Value));
    type Error = DecodeError<Take<R>>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let next = match phase {
                Phase2::A((r, (tag, wire_type))) => Phase2::B(self.decode_field(r, tag, wire_type)),
                Phase2::B((r, variant)) => {
                    match variant {
                        Variant3::A(v0) => self.value0 = Some(v0),
                        Variant3::B(v1) => self.value1 = Some(v1),
                        Variant3::C(()) => {}
                        Variant3::None => {}
                    }
                    if r.limit() == 0 {
                        let v0 = self.value0.take().expect("Never fails");
                        let v1 = self.value1.take().expect("Never fails");
                        return Ok(Async::Ready((r, (v0, v1))));
                    }
                    Phase2::A(DecodeTagAndWireType::new(r))
                }
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}

impl<R, A, B> Decode<R> for Message<(A, B)>
where
    R: Read,
    A: DecodeField<Take<R>>,
    B: DecodeField<Take<R>>,
{
    type Future = DecodeLengthDelimited<R, Message<(A, B)>>;
    fn decode(self, reader: R) -> Self::Future {
        LengthDelimited(self).decode(reader)
    }
}
impl<R, A, B> DecodePayload<R> for Message<(A, B)>
where
    R: Read,
    A: DecodeField<Take<R>>,
    B: DecodeField<Take<R>>,
{
    type Future = DecodeMessage2<R, A, B>;
    fn decode_payload(self, reader: Take<R>) -> Self::Future {
        DecodeMessage2::new(self, reader)
    }
}
