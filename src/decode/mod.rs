use std::io::Read;
use futures::Future;

use Error;
use traits::Field;
use wire::WireType;

pub mod futures;

mod fields;
mod types;
mod wires;

pub trait Decode<R: Read> {
    type Value: Default;
    type Future: Future<Item = (R, Self::Value), Error = Error<R>>;
    fn decode(reader: R) -> Self::Future;
}

pub trait DecodeField<R: Read>: Field {
    type Value: Default;
    type Future: Future<Item = (R, Self::Value), Error = Error<R>>;
    fn is_target(tag: u32) -> bool;
    fn decode_field(
        reader: R,
        tag: u32,
        wire_type: WireType,
        acc: Self::Value,
    ) -> Result<Self::Future, Error<R>>;
}
