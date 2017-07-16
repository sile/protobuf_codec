use std::io::{self, Write};
use futures::{Future, Poll, Async};

use {Error, ErrorKind};

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
