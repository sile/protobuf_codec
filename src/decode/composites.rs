use std::io::{Read, Take};
use std::mem;
use futures::{self, Future, Poll, Async};
use futures::future::{Either, Finished};

use {Type, Payload};
use composites::{Packed, Enum};
use decode::{Decode, DecodePayload, DecodeError};
use decode::futures::{DecodeLengthDelimited, DecodeVarint};
use wires::{Varint, LengthDelimited};

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
