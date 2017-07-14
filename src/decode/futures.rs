use std;
use std::io::{self, Read};
use std::mem;
use futures::{Future, Poll, Async};

pub use super::fields::{DecodePackedRepeated, DecodeOneof2};
pub use super::types::DecodeMessage2;
pub use super::wires::{DecodeVarint, DecodeMaybeVarint, DecodeLengthDelimited, DiscardWireValue};

use {Error, ErrorKind};
use wire::WireType;

pub struct DecodeTagAndWireType<R>(DecodeMaybeVarint<R>);
impl<R: Read> DecodeTagAndWireType<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeTagAndWireType(DecodeMaybeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeTagAndWireType<R> {
    type Item = (R, Option<(u32, WireType)>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            if let Some(n) = n {
                let tag = n >> 3;
                if tag > std::u32::MAX as u64 {
                    failed!(r, ErrorKind::Invalid, "Too large tag value: {}", tag);
                }
                let tag = tag as u32;
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
                Ok(Async::Ready((r, Some((tag, wire_type)))))
            } else {
                Ok(Async::Ready((r, None)))
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}

#[derive(Debug)]
pub struct VecPush<R, F, T>
where
    F: Future<Item = (R, T), Error = Error<R>>,
{
    future: F,
    vec: Vec<T>,
}
impl<R, F, T> VecPush<R, F, T>
where
    F: Future<Item = (R, T), Error = Error<R>>,
{
    pub(crate) fn new(vec: Vec<T>, future: F) -> Self {
        VecPush { future, vec }
    }
}
impl<R, F, T> Future for VecPush<R, F, T>
where
    F: Future<Item = (R, T), Error = Error<R>>,
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
    type Error = Error<R>;
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
    type Error = Error<R>;
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

// #[derive(Debug)]
// pub struct ReadAllBytes<R> {
//     reader: Option<R>,
//     bytes: Vec<u8>,
//     offset: usize,
// }
// impl<R> ReadAllBytes<R> {
//     pub fn new(reader: R) -> Self {
//         ReadAllBytes {
//             reader: Some(reader),
//             bytes: vec![0; 64],
//             offset: 0,
//         }
//     }
//     pub fn with_capacity(reader: R, capacity: usize) -> Self {
//         ReadAllBytes {
//             reader: Some(reader),
//             bytes: vec![0; capacity],
//             offset: 0,
//         }
//     }
// }
// impl<R: Read> Future for ReadAllBytes<R> {
//     type Item = (R, Vec<u8>);
//     type Error = Error<R>;
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         let mut r = self.reader.take().expect("Cannot poll ReadAllBytes twice");
//         loop {
//             if self.offset == self.bytes.len() {
//                 self.bytes.resize(self.offset * 2, 0);
//             }
//             match r.read(&mut self.bytes[self.offset..]) {
//                 Err(e) => {
//                     if e.kind() != io::ErrorKind::WouldBlock {
//                         failed_by_error!(r, ErrorKind::Other, e);
//                     }
//                     self.reader = Some(r);
//                     return Ok(Async::NotReady);
//                 }
//                 Ok(0) => {
//                     let mut bytes = mem::replace(&mut self.bytes, Vec::new());
//                     bytes.truncate(self.offset);
//                     return Ok(Async::Ready((r, bytes)));
//                 }
//                 Ok(read_size) => {
//                     self.offset += read_size;
//                 }
//             }
//         }
//     }
// }
