use std::io::Read;
use futures::{Future, Poll, Async};

use ErrorKind;
use wires;
use decode::{Decode, DecodePayload, DecodeError};
use decode::futures::{ReadByte, ReadBytes, Phase2};

impl<R: Read> Decode<R> for wires::Bit32 {
    type Future = ReadBytes<R, [u8; 4]>;
    fn decode(self, reader: R) -> Self::Future {
        ReadBytes::new(reader, [0; 4])
    }
}

impl<R: Read> Decode<R> for wires::Bit64 {
    type Future = ReadBytes<R, [u8; 8]>;
    fn decode(self, reader: R) -> Self::Future {
        ReadBytes::new(reader, [0; 8])
    }
}

#[derive(Debug)]
pub struct DecodeVarint<R> {
    value: u64,
    bits: usize,
    future: ReadByte<R>,
}
impl<R> DecodeVarint<R> {
    fn new(reader: R) -> Self {
        DecodeVarint {
            value: 0,
            bits: 0,
            future: ReadByte::new(reader),
        }
    }
}
impl<R: Read> Future for DecodeVarint<R> {
    type Item = (R, u64);
    type Error = DecodeError<R>;
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
impl<R: Read> Decode<R> for wires::Varint {
    type Future = DecodeVarint<R>;
    fn decode(self, reader: R) -> Self::Future {
        DecodeVarint::new(reader)
    }
}

#[derive(Debug)]
pub struct DecodeLengthDelimited<R, T>
where
    R: Read,
    T: DecodePayload<R>,
{
    phase: Phase2<DecodeVarint<R>, T::Future>,
    value: Option<T>,
}
impl<R, T> Future for DecodeLengthDelimited<R, T>
where
    R: Read,
    T: DecodePayload<R>,
{
    type Item = (R, T::Value);
    type Error = DecodeError<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let next = match phase {
                Phase2::A((r, len)) => {
                    let value = self.value.take().expect("Never fails");
                    Phase2::B(value.decode_payload(r.take(len)))
                }
                Phase2::B((r, value)) => return Ok(Async::Ready((r.into_inner(), value))),
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}
impl<R, T> Decode<R> for wires::LengthDelimited<T>
where
    R: Read,
    T: DecodePayload<R>,
{
    type Future = DecodeLengthDelimited<R, T>;
    fn decode(self, reader: R) -> Self::Future {
        let phase = Phase2::A(wires::Varint.decode(reader));
        let value = Some(self.0);
        DecodeLengthDelimited { phase, value }
    }
}
