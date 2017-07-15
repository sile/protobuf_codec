extern crate byteorder;
extern crate futures;
#[macro_use]
extern crate trackable;

pub use decode::Decode;
pub use types::Message;

pub use error::{Error, ErrorKind};

// pub mod composites;
// pub mod encode;
// pub mod fields;
// pub mod scalars;
// pub mod variants;
// pub mod wires;

// // TODO
// pub mod temp;

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct Tag(pub u32);

// pub trait Payload {
//     type Value: Default; // TODO
// }

// pub trait Type {
//     type Value: Default;
//     fn wire_type() -> WireType;
// }

// pub trait Field {
//     type Value: Default;
// }

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

pub mod decode;
pub mod fields;
pub mod traits;
pub mod tags;
pub mod types;
pub mod variants;
pub mod wire;

mod error;
mod util_futures;
