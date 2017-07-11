use std::io::Read;
use futures::Future;

pub use self::error::{Error, ErrorKind};

pub mod futures;
pub mod types;

mod error;

pub trait Decode<R: Read> {
    type Value;
    type Future: Future<Item = (R, Self::Value), Error = Error<R>>;
    fn decode(reader: R) -> Self::Future;
}
