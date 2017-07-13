use std;
use std::io::{self, Read};
use std::mem;
use futures::{Future, Poll, Async};
use trackable::Trackable;

pub use super::composites::{DecodePacked, DecodeMessage2};
pub use super::scalars::{DecodeBool, DecodeUint32, DecodeInt32, DecodeInt64, DecodeSint32,
                         DecodeSint64, DecodeFixed32, DecodeFixed64, DecodeSfixed32,
                         DecodeSfixed64, DecodeFloat, DecodeDouble, DecodeBytes, DecodeStr};
pub use super::variants::{DecodeVariant2, DecodeVariant3};
pub use super::wires::{DecodeVarint, DecodeLengthDelimited};

use {Tag, WireType, ErrorKind};
use decode::{Decode, DecodeError};
use scalars;
use wires::Varint;

// TODO: optimize
#[derive(Debug)]
pub enum Ignore<R: Read> {
    Varint(DecodeVarint<R>),
    Bit32(ReadBytes<R, [u8; 4]>),
    Bit64(ReadBytes<R, [u8; 8]>),
    Bytes(DecodeBytes<R>),
}
impl<R: Read> Ignore<R> {
    pub fn new(reader: R, wire_type: WireType) -> Self {
        match wire_type {
            WireType::Varint => Ignore::Varint(Varint.decode(reader)),
            WireType::Bit32 => Ignore::Bit32(ReadBytes::new(reader, [0; 4])),
            WireType::Bit64 => Ignore::Bit64(ReadBytes::new(reader, [0; 8])),
            WireType::LengthDelimited => Ignore::Bytes(scalars::Bytes.decode(reader)),
        }
    }
}
impl<R: Read> Future for Ignore<R> {
    type Item = (R, ());
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Ignore::Varint(ref mut f) => track!(f.poll())?.map(|(r, _)| (r, ())),
            Ignore::Bit32(ref mut f) => track!(f.poll())?.map(|(r, _)| (r, ())),
            Ignore::Bit64(ref mut f) => track!(f.poll())?.map(|(r, _)| (r, ())),
            Ignore::Bytes(ref mut f) => track!(f.poll())?.map(|(r, _)| (r, ())),
        })
    }
}

#[derive(Debug)]
pub enum Phase2<A, B> {
    A(A),
    B(B),
}
impl<A, B> Future for Phase2<A, B>
where
    A: Future,
    B: Future,
    A::Error: Trackable + From<B::Error>,
    B::Error: Trackable,
{
    type Item = Phase2<A::Item, B::Item>;
    type Error = A::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Phase2::A(ref mut f) => track!(f.poll())?.map(Phase2::A),
            Phase2::B(ref mut f) => track!(f.poll().map_err(From::from))?.map(Phase2::B),
        })
    }
}

#[derive(Debug)]
pub enum Phase3<A, B, C> {
    A(A),
    B(B),
    C(C),
}
impl<A, B, C> Future for Phase3<A, B, C>
where
    A: Future,
    B: Future,
    C: Future,
    A::Error: Trackable + From<B::Error> + From<C::Error>,
    B::Error: Trackable,
    C::Error: Trackable,
{
    type Item = Phase3<A::Item, B::Item, C::Item>;
    type Error = A::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Phase3::A(ref mut f) => track!(f.poll())?.map(Phase3::A),
            Phase3::B(ref mut f) => track!(f.poll().map_err(A::Error::from))?.map(Phase3::B),
            Phase3::C(ref mut f) => track!(f.poll().map_err(A::Error::from))?.map(Phase3::C),
        })
    }
}

pub struct DecodeTagAndWireType<R>(DecodeVarint<R>);
impl<R: Read> DecodeTagAndWireType<R> {
    pub fn new(reader: R) -> Self {
        DecodeTagAndWireType(Varint.decode(reader))
    }
}
impl<R: Read> Future for DecodeTagAndWireType<R> {
    type Item = (R, (Tag, WireType));
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            let tag = n >> 3;
            if tag > std::u32::MAX as u64 {
                failed!(r, ErrorKind::Invalid, "Too large tag value: {}", tag);
            }
            let tag = Tag(tag as u32);

            let wire_type = n & 0b111;
            let wire_type = match wire_type {
                0 => WireType::Varint,
                1 => WireType::Bit64,
                2 => WireType::LengthDelimited,
                3 | 4 => {
                    failed!(
                        r,
                        ErrorKind::Unsupported,
                        "Unsupported wire type: {}",
                        wire_type
                    )
                }
                5 => WireType::Bit32,
                _ => failed!(r, ErrorKind::Invalid, "Unknown wire type: {}", wire_type),
            };

            Ok(Async::Ready((r, (tag, wire_type))))
        } else {
            Ok(Async::NotReady)
        }
    }
}

#[derive(Debug)]
pub struct Push<R, F, T>
where
    F: Future<Item = (R, T), Error = DecodeError<R>>,
{
    future: F,
    vec: Vec<T>,
}
impl<R, F, T> Push<R, F, T>
where
    F: Future<Item = (R, T), Error = DecodeError<R>>,
{
    pub fn new(future: F, vec: Vec<T>) -> Self {
        Push { future, vec }
    }
}
impl<R, F, T> Future for Push<R, F, T>
where
    F: Future<Item = (R, T), Error = DecodeError<R>>,
{
    type Item = (R, Vec<T>);
    type Error = F::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, v)) = track!(self.future.poll())? {
            self.vec.push(v);
            let vec = mem::replace(&mut self.vec, Vec::new());
            Ok(Async::Ready((r, vec)))
        } else {
            Ok(Async::NotReady)
        }
    }
}

#[derive(Debug)]
pub struct Unused<R, T>(R, T);
impl<R, T> Future for Unused<R, T> {
    type Item = (R, T);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        unreachable!()
    }
}

#[derive(Debug)]
pub struct ReadByte<R> {
    reader: Option<R>,
}
impl<R> ReadByte<R> {
    pub fn new(reader: R) -> Self {
        ReadByte { reader: Some(reader) }
    }
}
impl<R: Read> Future for ReadByte<R> {
    type Item = (R, u8);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut r = self.reader.take().expect("Cannot poll ReadByte twice");
        let mut buf = [0; 1];
        match r.read(&mut buf) {
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    failed_by_error!(r, ErrorKind::Other, e);
                }
                self.reader = Some(r);
                Ok(Async::NotReady)
            }
            Ok(0) => {
                failed!(r, ErrorKind::UnexpectedEos);
            }
            Ok(_) => Ok(Async::Ready((r, buf[0]))),
        }
    }
}

#[derive(Debug)]
pub struct ReadBytes<R, B> {
    reader: Option<R>,
    bytes: Option<B>,
    offset: usize,
}
impl<R, B> ReadBytes<R, B> {
    pub fn new(reader: R, bytes: B) -> Self {
        ReadBytes {
            reader: Some(reader),
            bytes: Some(bytes),
            offset: 0,
        }
    }
}
impl<R: Read, B: AsMut<[u8]>> Future for ReadBytes<R, B> {
    type Item = (R, B);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut r = self.reader.take().expect("Cannot poll ReadBytes twice");
        let mut bytes = self.bytes.take().expect("Never fails");
        loop {
            match r.read(&mut bytes.as_mut()[self.offset..]) {
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        failed_by_error!(r, ErrorKind::Other, e);
                    }
                    self.reader = Some(r);
                    self.bytes = Some(bytes);
                    return Ok(Async::NotReady);
                }
                Ok(read_size) => {
                    self.offset += read_size;
                    if self.offset == bytes.as_mut().len() {
                        return Ok(Async::Ready((r, bytes)));
                    } else if read_size == 0 {
                        failed!(r, ErrorKind::UnexpectedEos);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ReadAllBytes<R> {
    reader: Option<R>,
    bytes: Vec<u8>,
    offset: usize,
}
impl<R> ReadAllBytes<R> {
    pub fn new(reader: R) -> Self {
        ReadAllBytes {
            reader: Some(reader),
            bytes: vec![0; 64],
            offset: 0,
        }
    }
    pub fn with_capacity(reader: R, capacity: usize) -> Self {
        ReadAllBytes {
            reader: Some(reader),
            bytes: vec![0; capacity],
            offset: 0,
        }
    }
}
impl<R: Read> Future for ReadAllBytes<R> {
    type Item = (R, Vec<u8>);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut r = self.reader.take().expect("Cannot poll ReadAllBytes twice");
        loop {
            if self.offset == self.bytes.len() {
                self.bytes.resize(self.offset * 2, 0);
            }
            match r.read(&mut self.bytes[self.offset..]) {
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        failed_by_error!(r, ErrorKind::Other, e);
                    }
                    self.reader = Some(r);
                    return Ok(Async::NotReady);
                }
                Ok(0) => {
                    let mut bytes = mem::replace(&mut self.bytes, Vec::new());
                    bytes.truncate(self.offset);
                    return Ok(Async::Ready((r, bytes)));
                }
                Ok(read_size) => {
                    self.offset += read_size;
                }
            }
        }
    }
}
