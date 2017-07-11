use std;
use std::io::{self, Read};
use byteorder::{ByteOrder, LittleEndian};
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

pub struct DecodeBool<R>(DecodeVarint<R>);
impl<R> DecodeBool<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeBool(DecodeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeBool<R> {
    type Item = (R, bool);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, n != 0)))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeFixed32<R>(ReadBytes<R, [u8; 4]>);
impl<R> DecodeFixed32<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeFixed32(ReadBytes::new(reader, [0; 4]))
    }
}
impl<R: Read> Future for DecodeFixed32<R> {
    type Item = (R, u32);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_u32(&bytes[..]))))
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
                    self.reader = Some(r);
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

pub struct ReadBytes<R, B> {
    reader: Option<R>,
    bytes: Option<B>,
    offset: usize,
}
impl<R, B> ReadBytes<R, B> {
    pub(crate) fn new(reader: R, bytes: B) -> Self {
        ReadBytes {
            reader: Some(reader),
            bytes: Some(bytes),
            offset: 0,
        }
    }
}
impl<R: Read, B: AsMut<[u8]>> Future for ReadBytes<R, B> {
    type Item = (R, B);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut r = self.reader.take().expect("Cannot poll ReadBytes twice");
        let mut bytes = self.bytes.take().expect("Never fails");
        loop {
            match r.read(&mut bytes.as_mut()[self.offset..]) {
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        self.reader = Some(r);
                        self.bytes = Some(bytes);
                        return Ok(Async::NotReady);
                    } else {
                        return Err(track!(Error::new(r, ErrorKind::Other.cause(e))));
                    }
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
