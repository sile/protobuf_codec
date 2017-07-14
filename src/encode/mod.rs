use std::io::Write;
use futures::Future;

macro_rules! failed {
    ($reader:expr, $kind:expr) => {
        {
            use trackable::error::ErrorKindExt;
            return Err(track!(::encode::EncodeError::new($reader, $kind.error().into())));
        }
    };
    ($reader:expr, $kind:expr, $($arg:tt),*) => {
        {
            use trackable::error::ErrorKindExt;
            return Err(track!(::encode::EncodeError::new($reader, $kind.cause(format!($($arg),*)).into())));
        }
    }
}
macro_rules! failed_by_error {
    ($reader:expr, $kind:expr, $cause:expr) => {
        {
            use trackable::error::ErrorKindExt;
            return Err(track!(::encode::EncodeError::new($reader, $kind.cause($cause).into())));
        }
    }
}

pub use self::error::EncodeError;

pub mod futures;

mod composites;
mod error;
mod fields;
mod scalars;
mod variants;
mod wires;

pub trait Encode<W: Write> {
    type Value;
    type Future: Future<Item = W, Error = EncodeError<W>>;
    fn encode(value: Self::Value, writer: W) -> Self::Future;
    fn encoded_size(value: &Self::Value) -> u64;
}
impl<W: Write> Encode<W> for Vec<u8> {
    type Value = Self;
    type Future = futures::WriteBytes<W, Vec<u8>>;
    fn encode(value: Self::Value, writer: W) -> Self::Future {
        futures::WriteBytes::new(value, writer)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        value.len() as u64
    }
}
