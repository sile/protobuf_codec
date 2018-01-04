use std::io::Read;
use std::marker::PhantomData;
use futures::{Async, Future, Poll};

use {Decode, Error};
use traits::TryFrom;

#[derive(Debug)]
pub struct DecodeInto<R, F, T>
where
    R: Read,
    F: Decode<R>,
{
    future: F::Future,
    _phantom: PhantomData<T>,
}
impl<R, F, T> DecodeInto<R, F, T>
where
    R: Read,
    T: From<F>,
    F: Decode<R>,
{
    pub fn new(reader: R) -> Self {
        Self::with_future(F::decode(reader))
    }
    pub fn with_future(future: F::Future) -> Self {
        DecodeInto {
            future,
            _phantom: PhantomData,
        }
    }
}
impl<R, F, T> Future for DecodeInto<R, F, T>
where
    R: Read,
    T: From<F>,
    F: Decode<R>,
{
    type Item = (R, T);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(track!(self.future.poll())?.map(|(r, v)| (r, From::from(v))))
    }
}

#[derive(Debug)]
pub struct DecodeTryInto<R, F, T>
where
    R: Read,
    F: Decode<R>,
{
    future: F::Future,
    _phantom: PhantomData<T>,
}
impl<R, F, T> DecodeTryInto<R, F, T>
where
    R: Read,
    T: TryFrom<F>,
    F: Decode<R>,
{
    pub fn new(reader: R) -> Self {
        DecodeTryInto {
            future: F::decode(reader),
            _phantom: PhantomData,
        }
    }
}
impl<R, F, T> Future for DecodeTryInto<R, F, T>
where
    R: Read,
    T: TryFrom<F>,
    F: Decode<R>,
{
    type Item = (R, T);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready((r, v)) = track!(self.future.poll())? {
            match track!(T::try_from(v)) {
                Err(e) => Err(Error {
                    stream: r,
                    error: e,
                }),
                Ok(v) => Ok(Async::Ready((r, v))),
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}
