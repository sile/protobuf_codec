use std::io::Write;
use futures::{Future, Poll, Async};

use decode::futures::Phase2; // TODO
use wires;
use super::{Encode, EncodeError};
use super::futures::{WriteByte, WriteBytes};

#[derive(Debug)]
pub struct EncodeVarint<W> {
    value: u64,
    future: WriteByte<W>,
}
impl<W> EncodeVarint<W> {
    fn new(mut value: u64, writer: W) -> Self {
        let mut b = value & 0b0111_1111;
        value >>= 7;
        if value != 0 {
            b |= 0b1000_0000;
        }
        let future = WriteByte::new(b as u8, writer);
        EncodeVarint { value, future }
    }
}
impl<W: Write> Future for EncodeVarint<W> {
    type Item = W;
    type Error = EncodeError<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(w) = track!(self.future.poll())? {
            if self.value == 0 {
                return Ok(Async::Ready(w));
            }
            let mut b = self.value & 0b0111_1111;
            self.value >>= 7;
            if self.value != 0 {
                b |= 0b1000_0000;
            }
            self.future = WriteByte::new(b as u8, w);
        }
        Ok(Async::NotReady)
    }
}
impl<W: Write> Encode<W> for wires::Varint {
    type Value = u64;
    type Future = EncodeVarint<W>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        EncodeVarint::new(value, writer)
    }
    fn encoded_size(value: &u64) -> u64 {
        let mut n = *value;
        for i in 1.. {
            n >>= 7;
            if n == 0 {
                return i;
            }
        }
        unreachable!()
    }
}

impl<W: Write> Encode<W> for wires::Bit32 {
    type Value = [u8; 4];
    type Future = WriteBytes<W, [u8; 4]>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        WriteBytes::new(value, writer)
    }
    fn encoded_size(_value: &[u8; 4]) -> u64 {
        4
    }
}

impl<W: Write> Encode<W> for wires::Bit64 {
    type Value = [u8; 8];
    type Future = WriteBytes<W, [u8; 8]>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        WriteBytes::new(value, writer)
    }
    fn encoded_size(_value: &[u8; 8]) -> u64 {
        8
    }
}

#[derive(Debug)]
pub struct EncodeLengthDelimited<W, T>
where
    W: Write,
    T: Encode<W>,
{
    value: Option<T::Value>, // TODO: `WithState`的なFutureを用意してそっちに乗せる
    phase: Phase2<EncodeVarint<W>, T::Future>,
}
impl<W, T> Future for EncodeLengthDelimited<W, T>
where
    W: Write,
    T: Encode<W>,
{
    type Item = W;
    type Error = EncodeError<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let next = match phase {
                Phase2::A(w) => {
                    let value = self.value.take().expect("Never fails");
                    Phase2::B(T::encode(value, w))
                }
                Phase2::B(w) => return Ok(Async::Ready(w)),
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}
impl<W, T> Encode<W> for wires::LengthDelimited<T>
where
    W: Write,
    T: Encode<W>,
{
    type Value = T::Value;
    type Future = EncodeLengthDelimited<W, T>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        let size = T::encoded_size(&value);
        let phase = Phase2::A(wires::Varint::encode(size, writer));
        EncodeLengthDelimited {
            value: Some(value),
            phase,
        }
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let size = T::encoded_size(value);
        <wires::Varint as Encode<W>>::encoded_size(&size) + size
    }
}
