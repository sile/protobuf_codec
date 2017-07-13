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
mod variants;
mod wires;

pub trait Decode<R: Read>: Type {
    type Future: Future<Item = (R, Self::Value), Error = DecodeError<R>>;
    fn decode(self, reader: R) -> Self::Future;
}

pub trait DecodePayload<R: Read>: Payload {
    type Future: Future<Item = (Take<R>, Self::Value), Error = DecodeError<Take<R>>>;
    fn decode_payload(self, reader: Take<R>) -> Self::Future;
}

pub trait DecodeField<R: Read>: Field {
    type Future: Future<Item = (R, Self::Value), Error = DecodeError<R>>;
    fn decode_field(
        self,
        reader: R,
        tag: Tag,
        wire_type: WireType,
        acc: Self::Value,
    ) -> DecodeFieldResult<Self::Future, R, Self::Value>;
}

#[derive(Debug)]
pub enum DecodeFieldResult<T, R, S> {
    Ok(T),
    Err(DecodeError<R>),
    NotTarget(R, S),
}
impl<T, R, S> DecodeFieldResult<T, R, S> {
    pub fn map<F, U>(self, f: F) -> DecodeFieldResult<U, R, S>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            DecodeFieldResult::Ok(v) => DecodeFieldResult::Ok(f(v)),
            DecodeFieldResult::Err(e) => DecodeFieldResult::Err(e),
            DecodeFieldResult::NotTarget(r, s) => DecodeFieldResult::NotTarget(r, s),
        }
    }
}
