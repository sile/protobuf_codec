extern crate bytecodec;
extern crate byteorder;
#[macro_use]
extern crate trackable;

macro_rules! try_encode {
    ($encoder:expr, $offset:expr, $buf:expr, $eos:expr) => {
        if !$encoder.is_idle() {
            $offset += track!($encoder.encode(&mut $buf[$offset..], $eos))?;
            if !$encoder.is_idle() {
                return Ok($offset);
            }
        }
    }
}

pub use tag::Tag;
pub use value::Value;

pub mod field;
pub mod message;
pub mod wire;

mod fields;
mod tag;
mod value;
