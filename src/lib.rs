//! Encoders and decoders for [Protocol Buffers][protobuf] based on [bytecodec] crate.
//!
//! # Examples
//!
//! TODO
//!
//! # References
//!
//! - [Protocol Buffers: Language Guide (proto2)][proto2]
//! - [Protocol Buffers: Language Guide (proto3)][proto3]
//! - [Protocol Buffers: Encoding][encoding]
//!
//! [bytecodec]: https://github.com/sile/bytecodec
//! [protobuf]: https://developers.google.com/protocol-buffers/docs/overview
//! [proto2]: https://developers.google.com/protocol-buffers/docs/proto
//! [proto3]: https://developers.google.com/protocol-buffers/docs/proto3
//! [encoding]: https://developers.google.com/protocol-buffers/docs/encoding
#![warn(missing_docs)]
#[macro_use]
extern crate bytecodec;
#[macro_use]
extern crate trackable;

pub mod field;
pub mod message;
pub mod scalar;
pub mod wire;

mod field_num;
mod fields;
mod oneof;
mod repeated_field;
mod value;
