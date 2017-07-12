use std::io::Read;
use futures::Future;

use {Tag, WireType};

pub use self::error::{Error, ErrorKind};

pub mod fields;
pub mod futures;
pub mod types;

mod error;

pub trait Decode<R: Read> {
    type Value;
    type Future: Future<Item = (R, Self::Value), Error = Error<R>>;
    fn decode(&self, reader: R) -> Self::Future;
}

pub trait FieldDecode<R: Read> {
    type Value: Default;
    type Future: Future<Item = (R, Self::Value), Error = Error<R>>;
    fn field_decode(
        &self,
        tag: Tag,
        wire_type: WireType,
        reader: R,
    ) -> FieldDecodeResult<Self::Future, R>;
}

pub enum FieldDecodeResult<T, R> {
    Ok(T),
    Err(Error<R>),
    NotSubject(R),
}
impl<T, R> FieldDecodeResult<T, R> {
    pub fn map<F, U>(self, f: F) -> FieldDecodeResult<U, R>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            FieldDecodeResult::Ok(v) => FieldDecodeResult::Ok(f(v)),
            FieldDecodeResult::Err(e) => FieldDecodeResult::Err(e),
            FieldDecodeResult::NotSubject(r) => FieldDecodeResult::NotSubject(r),
        }
    }
}
