use std::io::{Read, Take};
use futures::Future;

use {Tag, WireType, Type, Field, Payload};

macro_rules! failed {
    ($reader:expr, $kind:expr) => {
        {
            use trackable::error::ErrorKindExt;
            return Err(track!(::decode::DecodeError::new($reader, $kind.error().into())));
        }
    };
    ($reader:expr, $kind:expr, $($arg:tt),*) => {
        {
            use trackable::error::ErrorKindExt;
            return Err(track!(::decode::DecodeError::new($reader, $kind.cause(format!($($arg),*)).into())));
        }
    }
}
macro_rules! failed_by_error {
    ($reader:expr, $kind:expr, $cause:expr) => {
        {
            use trackable::error::ErrorKindExt;
            return Err(track!(::decode::DecodeError::new($reader, $kind.cause($cause).into())));
        }
    }
}

pub use self::error::DecodeError;

pub mod futures;

mod composites;
mod error;
mod fields;
mod scalars;
mod wires;

pub trait Decode<R: Read>: Type {
    type Future: Future<Item = (R, Self::Value), Error = DecodeError<R>>;
    fn decode(self, reader: R) -> Self::Future;
}

pub trait DecodePayload<R: Read>: Payload {
    type Future: Future<Item = (Take<R>, Self::Value), Error = DecodeError<Take<R>>>;
    fn decode_payload(self, reader: Take<R>) -> Self::Future;
}

pub trait FieldDecode<R: Read>: Field {
    type Future: Future<Item = (R, Self::Value), Error = DecodeError<R>>;
    fn field_decode(
        &self,
        reader: R,
        tag: Tag,
        wire_type: WireType,
        acc: Self::Value,
    ) -> FieldDecodeResult<Self::Future, R>;
}

#[derive(Debug)]
pub enum FieldDecodeResult<T, R> {
    Ok(T),
    Err(DecodeError<R>),
    NotTarget(R),
}
impl<T, R> FieldDecodeResult<T, R> {
    pub fn map<F, U>(self, f: F) -> FieldDecodeResult<U, R>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            FieldDecodeResult::Ok(v) => FieldDecodeResult::Ok(f(v)),
            FieldDecodeResult::Err(e) => FieldDecodeResult::Err(e),
            FieldDecodeResult::NotTarget(r) => FieldDecodeResult::NotTarget(r),
        }
    }
}
