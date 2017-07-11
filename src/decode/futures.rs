use std;
use std::io::Read;
use futures::{Future, Poll, Async};
use trackable::error::ErrorKindExt;

use {Tag, WireType};
use decode::{Error, ErrorKind};

macro_rules! failed {
    ($reader:expr, $kind:expr, $($arg:tt),*) => {
        return Err(track!(Error::new($reader, $kind.cause(format!($($arg),*)))));
    }
}

pub struct DecodeTagAndWireType<R>(DecodeVarint<R>);
impl<R> DecodeTagAndWireType<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeTagAndWireType(DecodeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeTagAndWireType<R> {
    type Item = (R, (Tag, WireType));
    type Error = Error<R>;
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

pub struct DecodeVarint<R>(R);
impl<R> DecodeVarint<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeVarint(reader)
    }
}
impl<R: Read> Future for DecodeVarint<R> {
    type Item = (R, u64);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        panic!()
    }
}
