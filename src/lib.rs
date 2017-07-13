extern crate byteorder;
extern crate futures;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};

pub mod composites;
pub mod decode;
pub mod fields;
pub mod scalars;
pub mod variants;
pub mod wires;

mod error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireType {
    Varint = 0,
    Bit32 = 5,
    Bit64 = 1,
    LengthDelimited = 2,
}

pub trait Payload {
    type Value: Default; // TODO
}

pub trait Type {
    type Value: Default;
    fn wire_type() -> WireType;
}

pub trait Field {
    type Value: Default;
}
