use std;
use std::io::{self, Read};
use std::mem;
use byteorder::{ByteOrder, LittleEndian};
use futures::{Future, Poll, Async};
use trackable::error::ErrorKindExt;

use {Tag, WireType};
use decode::{Error, ErrorKind, Decode};

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

pub struct DecodeUint32<R>(DecodeVarint<R>);
impl<R> DecodeUint32<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeUint32(DecodeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeUint32<R> {
    type Item = (R, u32);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            if n > std::u32::MAX as u64 {
                failed!(r, ErrorKind::Invalid, "Too large `uint32` value: {}", n);
            }
            Ok(Async::Ready((r, n as u32)))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeUint64<R>(DecodeVarint<R>);
impl<R> DecodeUint64<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeUint64(DecodeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeUint64<R> {
    type Item = (R, u64);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, n)))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeInt32<R>(DecodeVarint<R>);
impl<R> DecodeInt32<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeInt32(DecodeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeInt32<R> {
    type Item = (R, i32);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            let n = n as i64;
            if n > std::i32::MAX as i64 {
                failed!(r, ErrorKind::Invalid, "Too large `int32` value: {}", n);
            }
            if n < std::i32::MIN as i64 {
                failed!(r, ErrorKind::Invalid, "Too small `int32` value: {}", n);
            }
            Ok(Async::Ready((r, n as i32)))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeInt64<R>(DecodeVarint<R>);
impl<R> DecodeInt64<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeInt64(DecodeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeInt64<R> {
    type Item = (R, i64);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, n as i64)))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeSint32<R>(DecodeVarint<R>);
impl<R> DecodeSint32<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeSint32(DecodeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeSint32<R> {
    type Item = (R, i32);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            let n = ((n << 63) | (n >> 1)) as i64;
            if n > std::i32::MAX as i64 {
                failed!(r, ErrorKind::Invalid, "Too large `int32` value: {}", n);
            }
            if n < std::i32::MIN as i64 {
                failed!(r, ErrorKind::Invalid, "Too small `int32` value: {}", n);
            }
            Ok(Async::Ready((r, n as i32)))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeSint64<R>(DecodeVarint<R>);
impl<R> DecodeSint64<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeSint64(DecodeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeSint64<R> {
    type Item = (R, i64);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            let n = ((n << 63) | (n >> 1)) as i64;
            Ok(Async::Ready((r, n)))
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

pub struct DecodeSfixed32<R>(ReadBytes<R, [u8; 4]>);
impl<R> DecodeSfixed32<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeSfixed32(ReadBytes::new(reader, [0; 4]))
    }
}
impl<R: Read> Future for DecodeSfixed32<R> {
    type Item = (R, i32);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_i32(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeFloat<R>(ReadBytes<R, [u8; 4]>);
impl<R> DecodeFloat<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeFloat(ReadBytes::new(reader, [0; 4]))
    }
}
impl<R: Read> Future for DecodeFloat<R> {
    type Item = (R, f32);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_f32(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeFixed64<R>(ReadBytes<R, [u8; 8]>);
impl<R> DecodeFixed64<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeFixed64(ReadBytes::new(reader, [0; 8]))
    }
}
impl<R: Read> Future for DecodeFixed64<R> {
    type Item = (R, u64);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_u64(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeSfixed64<R>(ReadBytes<R, [u8; 8]>);
impl<R> DecodeSfixed64<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeSfixed64(ReadBytes::new(reader, [0; 8]))
    }
}
impl<R: Read> Future for DecodeSfixed64<R> {
    type Item = (R, i64);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_i64(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeDouble<R>(ReadBytes<R, [u8; 8]>);
impl<R> DecodeDouble<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeDouble(ReadBytes::new(reader, [0; 8]))
    }
}
impl<R: Read> Future for DecodeDouble<R> {
    type Item = (R, f64);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_f64(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodeBytes<R>(DecodeBytesInner<R>);
impl<R> DecodeBytes<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeBytes(DecodeBytesInner::new(reader))
    }
}
impl<R: Read> Future for DecodeBytes<R> {
    type Item = (R, Vec<u8>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, bytes)))
        } else {
            Ok(Async::NotReady)
        }
    }
}

enum DecodeBytesInner<R> {
    Length(DecodeVarint<R>),
    Bytes(ReadBytes<R, Vec<u8>>),
}
impl<R> DecodeBytesInner<R> {
    pub fn new(reader: R) -> Self {
        DecodeBytesInner::Length(DecodeVarint::new(reader))
    }
}
impl<R: Read> Future for DecodeBytesInner<R> {
    type Item = (R, Vec<u8>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let next = match *self {
                DecodeBytesInner::Length(ref mut f) => {
                    if let Async::Ready((r, len)) = track!(f.poll())? {
                        if len > std::usize::MAX as u64 {
                            failed!(r, ErrorKind::Invalid, "Too large bytes length: {}", len);
                        }
                        DecodeBytesInner::Bytes(ReadBytes::new(r, vec![0; len as usize]))
                    } else {
                        break;
                    }
                }
                DecodeBytesInner::Bytes(ref mut f) => {
                    return track!(f.poll());
                }
            };
            *self = next;
        }
        Ok(Async::NotReady)
    }
}

pub struct DecodeUtf8<R>(DecodeBytes<R>);
impl<R> DecodeUtf8<R> {
    pub(crate) fn new(reader: R) -> Self {
        DecodeUtf8(DecodeBytes::new(reader))
    }
}
impl<R: Read> Future for DecodeUtf8<R> {
    type Item = (R, String);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            match String::from_utf8(bytes) {
                Err(e) => Err(track!(Error::new(r, ErrorKind::Invalid.cause(e)))),
                Ok(s) => Ok(Async::Ready((r, s))),
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct DecodePacked<R, T>(DecodePackedInner<R, T>)
where
    R: Read,
    T: Decode<io::Take<R>>;
impl<R: Read, T: Decode<io::Take<R>>> DecodePacked<R, T> {
    pub(crate) fn new(decoder: T, reader: R) -> Self {
        DecodePacked(DecodePackedInner::new(decoder, reader))
    }
}
impl<R: Read, T: Clone + Decode<io::Take<R>>> Future for DecodePacked<R, T> {
    type Item = (R, Vec<T::Value>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        track!(self.0.poll())
    }
}

enum DecodePackedInner<R, T>
where
    R: Read,
    T: Decode<io::Take<R>>,
{
    Length(T, DecodeVarint<R>),
    Fields(DecodePackedFields<R, T>),
}
impl<R: Read, T: Decode<io::Take<R>>> DecodePackedInner<R, T> {
    pub fn new(decoder: T, reader: R) -> Self {
        DecodePackedInner::Length(decoder, DecodeVarint::new(reader))
    }
}
impl<R: Read, T: Clone + Decode<io::Take<R>>> Future for DecodePackedInner<R, T> {
    type Item = (R, Vec<T::Value>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let next = match *self {
                DecodePackedInner::Length(ref decoder, ref mut f) => {
                    if let Async::Ready((r, len)) = track!(f.poll())? {
                        if len == 0 {
                            return Ok(Async::Ready((r, Vec::new())));
                        } else {
                            DecodePackedInner::Fields(
                                DecodePackedFields::new(decoder.clone(), r.take(len)),
                            )
                        }
                    } else {
                        break;
                    }
                }
                DecodePackedInner::Fields(ref mut f) => {
                    return track!(f.poll());
                }
            };
            *self = next;
        }
        Ok(Async::NotReady)
    }
}

struct DecodePackedFields<R, T>
where
    R: Read,
    T: Decode<io::Take<R>>,
{
    future: T::Future,
    decoder: T,
    fields: Vec<T::Value>,
}
impl<R: Read, T: Decode<io::Take<R>>> DecodePackedFields<R, T> {
    pub fn new(decoder: T, reader: io::Take<R>) -> Self {
        DecodePackedFields {
            future: decoder.decode(reader),
            decoder,
            fields: Vec::new(),
        }
    }
}
impl<R: Read, T: Decode<io::Take<R>>> Future for DecodePackedFields<R, T> {
    type Item = (R, Vec<T::Value>);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready((r, field)) = track!(self.future.poll())? {
            self.fields.push(field);
            if r.limit() == 0 {
                let item = (r.into_inner(), mem::replace(&mut self.fields, Vec::new()));
                return Ok(Async::Ready(item));
            }
            self.future = self.decoder.decode(r);
        }
        Ok(Async::NotReady)
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

struct ReadBytes<R, B> {
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
