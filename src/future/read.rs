use std::io::{self, Read};
use futures::{Future, Poll, Async};

use {Error, ErrorKind};

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
