use std::io::Write;
use futures::{Async, Future, Poll};

use {Encode, Error};
use future::util::{Phase2, WithState};
use future::write::{WriteByte, WriteBytes};
use wire::types::{Bit32, Bit64, LengthDelimited, Varint};

impl<W: Write> Encode<W> for Bit32 {
    type Future = WriteBytes<W, [u8; 4]>;
    fn encode(self, writer: W) -> Self::Future {
        WriteBytes::new(writer, self.0)
    }
    fn encoded_size(&self) -> u64 {
        4
    }
}

impl<W: Write> Encode<W> for Bit64 {
    type Future = WriteBytes<W, [u8; 8]>;
    fn encode(self, writer: W) -> Self::Future {
        WriteBytes::new(writer, self.0)
    }
    fn encoded_size(&self) -> u64 {
        8
    }
}

#[derive(Debug)]
pub struct EncodeVarint<W> {
    value: u64,
    future: WriteByte<W>,
}
impl<W> EncodeVarint<W> {
    fn new(writer: W, value: &Varint) -> Self {
        let mut value = value.0;
        let mut b = value & 0b0111_1111;
        value >>= 7;
        if value != 0 {
            b |= 0b1000_0000;
        }
        let future = WriteByte::new(writer, b as u8);
        EncodeVarint { value, future }
    }
}
impl<W: Write> Future for EncodeVarint<W> {
    type Item = W;
    type Error = Error<W>;
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
            self.future = WriteByte::new(w, b as u8);
        }
        Ok(Async::NotReady)
    }
}
impl<W: Write> Encode<W> for Varint {
    type Future = EncodeVarint<W>;
    fn encode(self, writer: W) -> Self::Future {
        EncodeVarint::new(writer, &self)
    }
    fn encoded_size(&self) -> u64 {
        for i in 1.. {
            if (self.0 >> (i * 7)) == 0 {
                return i;
            }
        }
        unreachable!()
    }
}

#[derive(Debug)]
pub struct EncodeLengthDelimited<W, T>
where
    W: Write,
    T: Encode<W>,
{
    phase: Phase2<WithState<EncodeVarint<W>, T>, T::Future>,
}
impl<W, T> Future for EncodeLengthDelimited<W, T>
where
    W: Write,
    T: Encode<W>,
{
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let next = match phase {
                Phase2::A((w, v)) => Phase2::B(v.encode(w)),
                Phase2::B(w) => return Ok(Async::Ready(w)),
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}
impl<W, T> Encode<W> for LengthDelimited<T>
where
    W: Write,
    T: Encode<W>,
{
    type Future = EncodeLengthDelimited<W, T>;
    fn encode(self, writer: W) -> Self::Future {
        let size = T::encoded_size(&self.0);
        let future = Varint(size).encode(writer);
        let phase = Phase2::A(WithState::new(future, self.0));
        EncodeLengthDelimited { phase }
    }
    fn encoded_size(&self) -> u64 {
        let size = T::encoded_size(&self.0);
        Encode::<W>::encoded_size(&Varint(size)) + size
    }
}
