use std::io::{self, Write};
use futures::{Future, Poll, Async};

// pub use super::composites::EncodeMessage2;
// pub use super::fields::{EncodeField, EncodeRepeated, EncodePackedRepeated};
// pub use super::variants::EncodeVariant2;
pub use super::wires::{EncodeVarint, EncodeLengthDelimited};

use {Error, ErrorKind};

// #[derive(Debug)]
// pub struct EncodeTagAndWireType<W>(EncodeVarint<W>);
// impl<W: Write> EncodeTagAndWireType<W> {
//     pub fn new(tag: Tag, wire_type: WireType, writer: W) -> Self {
//         let n = (tag.0 << 3) as u64 | wire_type as u64;
//         EncodeTagAndWireType(wires::Varint::encode(n, writer))
//     }
// }
// impl<W: Write> Future for EncodeTagAndWireType<W> {
//     type Item = W;
//     type Error = Error<W>;
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         track!(self.0.poll())
//     }
// }

#[derive(Debug)]
pub struct WriteByte<W> {
    writer: Option<W>,
    byte: u8,
}
impl<W> WriteByte<W> {
    pub fn new(writer: W, byte: u8) -> Self {
        WriteByte {
            byte,
            writer: Some(writer),
        }
    }
}
impl<W: Write> Future for WriteByte<W> {
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut writer = self.writer.take().expect("Cannot poll WriteByte twice");
        match writer.write(&[self.byte][..]) {
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    failed_by_error!(writer, ErrorKind::Other, e);
                }
                Ok(Async::NotReady)
            }
            Ok(0) => failed!(writer, ErrorKind::UnexpectedEos),
            Ok(_) => Ok(Async::Ready(writer)),
        }
    }
}

#[derive(Debug)]
pub struct WriteBytes<W, B> {
    writer: Option<W>,
    bytes: B,
    offset: usize,
}
impl<W, B> WriteBytes<W, B> {
    pub fn new(writer: W, bytes: B) -> Self {
        WriteBytes {
            writer: Some(writer),
            bytes,
            offset: 0,
        }
    }
}
impl<W: Write, B: AsRef<[u8]>> Future for WriteBytes<W, B> {
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut writer = self.writer.take().expect("Cannot poll WriteBytes twice");
        loop {
            match writer.write(&self.bytes.as_ref()[self.offset..]) {
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        failed_by_error!(writer, ErrorKind::Other, e);
                    }
                    return Ok(Async::NotReady);
                }
                Ok(written_size) => {
                    self.offset += written_size;
                    if self.offset == self.bytes.as_ref().len() {
                        return Ok(Async::Ready(writer));
                    }
                    if written_size == 0 {
                        failed!(writer, ErrorKind::UnexpectedEos)
                    }
                }
            }
        }
    }
}
