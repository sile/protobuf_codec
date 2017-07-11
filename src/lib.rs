extern crate byteorder;
extern crate futures;
#[macro_use]
extern crate trackable;

// pub use error::{Error, ErrorKind};

pub mod decode;
//pub mod decoder;

// mod error;

// pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireType {
    Varint = 0,
    Bit32 = 5,
    Bit64 = 1,
    LengthDelimited = 2,
}
