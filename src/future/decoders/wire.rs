use std::io::{self, Read, Take};
use futures::{Future, Poll, Async};

use {Error, ErrorKind, Decode};
// use util_futures::{UnwrapTake, Phase2, Phase4};
use future::readers::{ReadByte, ReadBytes};
use wire::WireType;
use wire::types::{Varint, Bit32, Bit64, LengthDelimited};

impl<R: Read> Decode<R> for Bit32 {
    type Future = ConvertFrom<ReadBytes<R, [u8; 4]>, Self>;
    fn decode(reader: R) -> Self::Future {
        ConvertFrom::new(ReadBytes::new(reader, [0; 4]))
    }
}

// impl<R: Read> Decode<R> for Bit64 {
//     type Future = ReadBytes<R, [u8; 8]>;
//     fn decode(reader: R) -> Self::Future {
//         ReadBytes::new(reader, [0; 8])
//     }
// }

// #[derive(Debug)]
// pub struct DecodeVarint<R> {
//     value: u64,
//     bits: usize,
//     future: ReadByte<R>,
// }
// impl<R> DecodeVarint<R> {
//     fn new(reader: R) -> Self {
//         DecodeVarint {
//             value: 0,
//             bits: 0,
//             future: ReadByte::new(reader),
//         }
//     }
// }
// impl<R: Read> Future for DecodeVarint<R> {
//     type Item = (R, u64);
//     type Error = Error<R>;
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         while let Async::Ready((r, b)) = track!(self.future.poll())? {
//             self.value += ((b & 0b0111_1111) as u64) << self.bits;
//             self.bits += 7;
//             let is_last = (b >> 7) == 0;
//             if is_last {
//                 return Ok(Async::Ready((r, self.value)));
//             } else if self.bits > 64 {
//                 failed!(r, ErrorKind::Invalid, "Too large Varint");
//             } else {
//                 self.future = ReadByte::new(r)
//             }
//         }
//         Ok(Async::NotReady)
//     }
// }
// impl<R: Read> Decode<R> for Varint {
//     type Future = DecodeVarint<R>;
//     fn decode(reader: R) -> Self::Future {
//         DecodeVarint::new(reader)
//     }
// }

// #[derive(Debug)]
// pub struct DecodeMaybeVarint<R>(DecodeVarint<R>);
// impl<R> DecodeMaybeVarint<R> {
//     pub fn new(reader: R) -> Self {
//         DecodeMaybeVarint(DecodeVarint::new(reader))
//     }
// }
// impl<R: Read> Future for DecodeMaybeVarint<R> {
//     type Item = (R, Option<u64>);
//     type Error = Error<R>;
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         match self.0.poll() {
//             Err(e) => {
//                 if *e.error.kind() == ErrorKind::UnexpectedEos && self.0.bits == 0 {
//                     Ok(Async::Ready((e.stream, None)))
//                 } else {
//                     Err(e)
//                 }
//             }
//             Ok(v) => Ok(v.map(|(r, v)| (r, Some(v)))),
//         }
//     }
// }

// #[derive(Debug)]
// pub struct DecodeLengthDelimited<R, T>
// where
//     R: Read,
//     T: Decode<Take<R>>,
// {
//     phase: Phase2<DecodeVarint<R>, UnwrapTake<T::Future>>,
// }
// impl<R, T> Future for DecodeLengthDelimited<R, T>
// where
//     R: Read,
//     T: Decode<Take<R>>,
// {
//     type Item = (R, T::Value);
//     type Error = Error<R>;
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         while let Async::Ready(phase) = track!(self.phase.poll())? {
//             let next = match phase {
//                 Phase2::A((r, len)) => Phase2::B(UnwrapTake(T::decode(r.take(len)))),
//                 Phase2::B((r, value)) => return Ok(Async::Ready((r.into_inner(), value))),
//             };
//             self.phase = next;
//         }
//         Ok(Async::NotReady)
//     }
// }
// impl<R, T> Decode<R> for LengthDelimited<T>
// where
//     R: Read,
//     T: Decode<Take<R>>,
// {
//     type Future = DecodeLengthDelimited<R, T>;
//     fn decode(reader: R) -> Self::Future {
//         let phase = Phase2::A(Varint::decode(reader));
//         DecodeLengthDelimited { phase }
//     }
// }

// #[derive(Debug)]
// pub struct DiscardWireValue<R: Read> {
//     phase: Phase4<
//         DecodeVarint<R>,
//         ReadBytes<R, [u8; 4]>,
//         ReadBytes<R, [u8; 8]>,
//         DecodeLengthDelimited<R, Null>,
//     >,
// }
// impl<R: Read> DiscardWireValue<R> {
//     pub fn new(reader: R, wire_type: WireType) -> Self {
//         let phase = match wire_type {
//             WireType::Varint => Phase4::A(DecodeVarint::new(reader)),
//             WireType::Bit32 => Phase4::B(ReadBytes::new(reader, [0; 4])),
//             WireType::Bit64 => Phase4::C(ReadBytes::new(reader, [0; 8])),
//             WireType::LengthDelimited => Phase4::D(LengthDelimited::<Null>::decode(reader)),
//         };
//         DiscardWireValue { phase }
//     }
// }
// impl<R: Read> Future for DiscardWireValue<R> {
//     type Item = (R, ());
//     type Error = Error<R>;
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         if let Async::Ready(phase) = track!(self.phase.poll())? {
//             Ok(Async::Ready(match phase {
//                 Phase4::A((r, _)) => (r, ()),
//                 Phase4::B((r, _)) => (r, ()),
//                 Phase4::C((r, _)) => (r, ()),
//                 Phase4::D((r, _)) => (r, ()),
//             }))
//         } else {
//             Ok(Async::NotReady)
//         }
//     }
// }

// #[derive(Debug)]
// struct Null;
// impl Pattern for Null {
//     type Value = ();
// }
// impl<R: Read> Decode<R> for Null {
//     type Future = DiscardAllBytes<R>;
//     fn decode(reader: R) -> Self::Future {
//         DiscardAllBytes { reader: Some(reader) }
//     }
// }

// #[derive(Debug)]
// struct DiscardAllBytes<R> {
//     reader: Option<R>,
// }
// impl<R: Read> Future for DiscardAllBytes<R> {
//     type Item = (R, ());
//     type Error = Error<R>;
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         let mut reader = self.reader.take().expect(
//             "Cannot poll DiscardAllBytes twice",
//         );
//         let mut buf = [0; 1024];
//         loop {
//             match reader.read(&mut buf) {
//                 Err(e) => {
//                     if e.kind() == io::ErrorKind::WouldBlock {
//                         self.reader = Some(reader);
//                         return Ok(Async::NotReady);
//                     }
//                     failed_by_error!(reader, ErrorKind::Other, e);
//                 }
//                 Ok(0) => break,
//                 Ok(_) => {}
//             }
//         }
//         Ok(Async::Ready((reader, ())))
//     }
// }
