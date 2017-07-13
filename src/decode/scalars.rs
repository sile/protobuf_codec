use std;
use std::io::{Read, Take};
use byteorder::{ByteOrder, LittleEndian};
use futures::{Future, Poll, Async};

use {Payload, ErrorKind};
use scalars;
use wires::{Varint, LengthDelimited};
use decode::{Decode, DecodePayload, DecodeError};
use decode::futures::{DecodeVarint, DecodeLengthDelimited, ReadBytes, ReadAllBytes};

#[derive(Debug)]
pub struct DecodeBool<R>(DecodeVarint<R>);
impl<R: Read> Future for DecodeBool<R> {
    type Item = (R, bool);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, n != 0)))
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R: Read> Decode<R> for scalars::Bool {
    type Future = DecodeBool<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeBool(Varint.decode(reader))
    }
}

#[derive(Debug)]
pub struct DecodeUint32<R>(DecodeVarint<R>);
impl<R: Read> Future for DecodeUint32<R> {
    type Item = (R, u32);
    type Error = DecodeError<R>;
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
impl<R: Read> Decode<R> for scalars::Uint32 {
    type Future = DecodeUint32<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeUint32(Varint.decode(reader))
    }
}
impl<R: Read> Decode<R> for scalars::Uint64 {
    type Future = DecodeVarint<R>;
    fn decode(self, reader: R) -> Self::Future {
        Varint.decode(reader)
    }
}

#[derive(Debug)]
pub struct DecodeInt32<R>(DecodeVarint<R>);
impl<R: Read> Future for DecodeInt32<R> {
    type Item = (R, i32);
    type Error = DecodeError<R>;
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
impl<R: Read> Decode<R> for scalars::Int32 {
    type Future = DecodeInt32<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeInt32(Varint.decode(reader))
    }
}

#[derive(Debug)]
pub struct DecodeInt64<R>(DecodeVarint<R>);
impl<R: Read> Future for DecodeInt64<R> {
    type Item = (R, i64);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, n as i64)))
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R: Read> Decode<R> for scalars::Int64 {
    type Future = DecodeInt64<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeInt64(Varint.decode(reader))
    }
}

#[derive(Debug)]
pub struct DecodeSint32<R>(DecodeVarint<R>);
impl<R: Read> Future for DecodeSint32<R> {
    type Item = (R, i32);
    type Error = DecodeError<R>;
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
impl<R: Read> Decode<R> for scalars::Sint32 {
    type Future = DecodeSint32<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeSint32(Varint.decode(reader))
    }
}

#[derive(Debug)]
pub struct DecodeSint64<R>(DecodeVarint<R>);
impl<R: Read> Future for DecodeSint64<R> {
    type Item = (R, i64);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, n)) = track!(self.0.poll())? {
            let n = ((n << 63) | (n >> 1)) as i64;
            Ok(Async::Ready((r, n)))
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R: Read> Decode<R> for scalars::Sint64 {
    type Future = DecodeSint64<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeSint64(Varint.decode(reader))
    }
}

#[derive(Debug)]
pub struct DecodeFixed32<R>(ReadBytes<R, [u8; 4]>);
impl<R: Read> Future for DecodeFixed32<R> {
    type Item = (R, u32);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_u32(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R: Read> Decode<R> for scalars::Fixed32 {
    type Future = DecodeFixed32<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeFixed32(ReadBytes::new(reader, [0; 4]))
    }
}

#[derive(Debug)]
pub struct DecodeFixed64<R>(ReadBytes<R, [u8; 8]>);
impl<R: Read> Future for DecodeFixed64<R> {
    type Item = (R, u64);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_u64(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R: Read> Decode<R> for scalars::Fixed64 {
    type Future = DecodeFixed64<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeFixed64(ReadBytes::new(reader, [0; 8]))
    }
}

#[derive(Debug)]
pub struct DecodeSfixed32<R>(ReadBytes<R, [u8; 4]>);
impl<R: Read> Future for DecodeSfixed32<R> {
    type Item = (R, i32);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_i32(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R: Read> Decode<R> for scalars::Sfixed32 {
    type Future = DecodeSfixed32<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeSfixed32(ReadBytes::new(reader, [0; 4]))
    }
}

#[derive(Debug)]
pub struct DecodeSfixed64<R>(ReadBytes<R, [u8; 8]>);
impl<R: Read> Future for DecodeSfixed64<R> {
    type Item = (R, i64);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_i64(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R: Read> Decode<R> for scalars::Sfixed64 {
    type Future = DecodeSfixed64<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeSfixed64(ReadBytes::new(reader, [0; 8]))
    }
}

#[derive(Debug)]
pub struct DecodeFloat<R>(ReadBytes<R, [u8; 4]>);
impl<R: Read> Future for DecodeFloat<R> {
    type Item = (R, f32);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_f32(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R: Read> Decode<R> for scalars::Float {
    type Future = DecodeFloat<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeFloat(ReadBytes::new(reader, [0; 4]))
    }
}

#[derive(Debug)]
pub struct DecodeDouble<R>(ReadBytes<R, [u8; 8]>);
impl<R: Read> Future for DecodeDouble<R> {
    type Item = (R, f64);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            Ok(Async::Ready((r, LittleEndian::read_f64(&bytes[..]))))
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R: Read> Decode<R> for scalars::Double {
    type Future = DecodeDouble<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeDouble(ReadBytes::new(reader, [0; 8]))
    }
}

#[derive(Debug)]
pub struct DecodeBytes<R: Read>(DecodeLengthDelimited<R, AllBytes>);
impl<R: Read> Future for DecodeBytes<R> {
    type Item = (R, Vec<u8>);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        track!(self.0.poll())
    }
}
impl<R: Read> Decode<R> for scalars::Bytes {
    type Future = DecodeBytes<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeBytes(LengthDelimited(AllBytes).decode(reader))
    }
}

#[derive(Debug)]
pub struct DecodeStr<R: Read>(DecodeBytes<R>);
impl<R: Read> Future for DecodeStr<R> {
    type Item = (R, String);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, bytes)) = track!(self.0.poll())? {
            match String::from_utf8(bytes) {
                Err(e) => failed_by_error!(r, ErrorKind::Invalid, e),
                Ok(s) => Ok(Async::Ready((r, s))),
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}
impl<R: Read> Decode<R> for scalars::Str {
    type Future = DecodeStr<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeStr(scalars::Bytes.decode(reader))
    }
}

#[derive(Debug)]
struct AllBytes;
impl Payload for AllBytes {
    type Value = Vec<u8>;
}
impl<R: Read> DecodePayload<R> for AllBytes {
    type Future = ReadAllBytes<Take<R>>;
    fn decode_payload(self, reader: Take<R>) -> Self::Future {
        let capacity = reader.limit() as usize;
        ReadAllBytes::with_capacity(reader, capacity)
    }
}
