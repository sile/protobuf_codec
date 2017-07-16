extern crate byteorder;
extern crate futures;
#[macro_use]
extern crate trackable;

pub use traits::{Decode, Encode};
pub use message::Message;

pub use error::{Error, ErrorKind};

macro_rules! failed {
    ($stream:expr, $kind:expr) => {
        {
            use trackable::error::ErrorKindExt;
            return Err(track!(::Error{stream: $stream, error: $kind.error().into()}));
        }
    };
    ($stream:expr, $kind:expr, $($arg:tt),*) => {
        {
            use trackable::error::ErrorKindExt;
            let error = $kind.cause(format!($($arg),*)).into();
            return Err(track!(::Error{stream: $stream, error}));
        }
    }
}
macro_rules! failed_by_error {
    ($stream:expr, $kind:expr, $cause:expr) => {
        {
            use trackable::error::ErrorKindExt;
            return Err(track!(::Error{stream: $stream, error: $kind.cause($cause).into()}));
        }
    }
}

pub mod fields;
pub mod future;
pub mod traits;
pub mod tags;
pub mod types;
pub mod variants;
pub mod wire;

mod error;
mod message;

pub type Result<T> = std::result::Result<T, trackable::error::TrackableError<ErrorKind>>;
