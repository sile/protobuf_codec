#[macro_use]
extern crate bytecodec;
#[macro_use]
extern crate trackable;

pub mod field;
pub mod message;
pub mod scalar;
pub mod tag;
pub mod wire;

mod field_encoder; // TODO: delete
mod fields;
