use std;
use std::io::{self, Read};
use futures::{Future, Poll, Async};
use trackable::error::ErrorKindExt;

use {Tag, WireType};
use decode::{Error, ErrorKind};

macro_rules! failed {
    ($reader:expr, $kind:expr) => {
        return Err(track!(Error::new($reader, $kind.error())));
    };
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

pub struct DecodeVarint<R> {
    value: u64,
    bits: usize,
    future: ReadByte<R>,
}
impl<R> DecodeVarint<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeVarint {
            value: 0,
            bits: 0,
            future: ReadByte::new(reader),
        }
    }
}
impl<R: Read> Future for DecodeVarint<R> {
    type Item = (R, u64);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready((r, b)) = track!(self.future.poll())? {
            self.value += ((b & 0b0111_1111) as u64) << self.bits;
            self.bits += 7;
            let is_last = (b >> 7) == 0;
            if is_last {
                return Ok(Async::Ready((r, self.value)));
            } else if self.bits > 64 {
                failed!(r, ErrorKind::Invalid, "Too large Varint");
            } else {
                self.future = ReadByte::new(r)
            }
        }
        Ok(Async::NotReady)
    }
}

struct ReadByte<R> {
    reader: Option<R>,
}
impl<R> ReadByte<R> {
    pub fn new(reader: R) -> Self {
        ReadByte { reader: Some(reader) }
    }
}
impl<R: Read> Future for ReadByte<R> {
    type Item = (R, u8);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut r = self.reader.take().expect("Cannot poll ReadByte twice");
        let mut buf = [0; 1];
        match r.read(&mut buf) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(track!(Error::new(r, ErrorKind::Other.cause(e))))
                }
            }
            Ok(0) => {
                failed!(r, ErrorKind::UnexpectedEos);
            }
            Ok(_) => Ok(Async::Ready((r, buf[0]))),
        }
    }
}
