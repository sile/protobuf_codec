//! TODO
//!
//! # References
//!
//! - [Protocol Buffers: Language Guide (proto2)][proto2]
//! - [Protocol Buffers: Language Guide (proto3)][proto3]
//! - [Protocol Buffers: Encoding][encoding]
//!
//! [proto2]: https://developers.google.com/protocol-buffers/docs/proto
//! [proto3]: https://developers.google.com/protocol-buffers/docs/proto3
//! [encoding]: https://developers.google.com/protocol-buffers/docs/encoding
#[macro_use]
extern crate bytecodec;
#[macro_use]
extern crate trackable;

pub mod field;
pub mod message;
pub mod scalar;
pub mod tag;
pub mod value;
pub mod wire;

mod fields;
mod oneof;
mod repeated_field;
