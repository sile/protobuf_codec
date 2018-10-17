//! `package protobuf_codec.protobuf.trackable;`
//!
//! See also [protobuf_codec/protobuf/trackable.proto].
//!
//! [protobuf_codec/protobuf/trackable.proto]: https://github.com/sile/protobuf_codec/blob/master/protobuf/trackable.proto
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
use bytecodec::{ByteCount, Decode, Encode, Eos, Result, SizedEncode};
use std::error::Error;
use trackable::error::{ErrorKindExt, TrackableError};
use trackable::{Location, Trackable};

use field::num::{F1, F2, F3, F4};
use field::{
    FieldDecoder, FieldEncoder, Fields, MaybeDefault, MessageFieldDecoder, MessageFieldEncoder,
    Repeated,
};
use message::{MessageDecode, MessageDecoder, MessageEncode, MessageEncoder};
use scalar::{StringDecoder, StringEncoder, Uint32Decoder, Uint32Encoder};

/// Decoder for [TrackableError].
///
/// See also [trackable.proto].
///
/// [TrackableError]: https://docs.rs/trackable/0.2/trackable/error/struct.TrackableError.html
/// [trackable.proto]: https://github.com/sile/protobuf_codec/blob/master/protobuf/trackable.proto
#[derive(Debug, Default)]
pub struct ErrorDecoder {
    inner: MessageDecoder<
        Fields<(
            MaybeDefault<FieldDecoder<F1, StringDecoder>>,
            MaybeDefault<FieldDecoder<F2, StringDecoder>>,
            Repeated<MessageFieldDecoder<F3, LocationDecoder>, Vec<Location>>,
        )>,
    >,
}
impl ErrorDecoder {
    /// Makes a new `ErrorDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Decode for ErrorDecoder {
    type Item = TrackableError<String>;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.inner.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let (kind, cause, locations) = track!(self.inner.finish_decoding())?;
        let mut e = if cause.is_empty() {
            kind.error()
        } else {
            kind.cause(cause)
        };
        if let Some(h) = e.history_mut() {
            for l in locations {
                h.add(l);
            }
        }
        Ok(e)
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }
}
impl MessageDecode for ErrorDecoder {}

/// Encoder for [TrackableError].
///
/// See also [trackable.proto].
///
/// [TrackableError]: https://docs.rs/trackable/0.2/trackable/error/struct.TrackableError.html
/// [trackable.proto]: https://github.com/sile/protobuf_codec/blob/master/protobuf/trackable.proto
#[derive(Debug, Default)]
pub struct ErrorEncoder {
    inner: MessageEncoder<
        Fields<(
            MaybeDefault<FieldEncoder<F1, StringEncoder>>,
            MaybeDefault<FieldEncoder<F2, StringEncoder>>,
            Repeated<MessageFieldEncoder<F3, LocationEncoder>, Vec<Location>>,
        )>,
    >,
}
impl ErrorEncoder {
    /// Makes a new `ErrorEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Encode for ErrorEncoder {
    type Item = TrackableError<String>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.inner.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let item = (
            item.kind().clone(),
            item.cause()
                .map(|e| e.to_string())
                .unwrap_or_else(String::new),
            item.history()
                .map(|h| h.events().to_owned())
                .unwrap_or_else(Vec::new),
        );
        track!(self.inner.start_encoding(item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }
}
impl MessageEncode for ErrorEncoder {}

/// Decoder for [Location].
///
/// See also [trackable.proto].
///
/// [Location]: https://docs.rs/trackable/0.2/trackable/struct.Location.html
/// [trackable.proto]: https://github.com/sile/protobuf_codec/blob/master/protobuf/trackable.proto
#[derive(Debug, Default)]
pub struct LocationDecoder {
    inner: MessageDecoder<
        Fields<(
            MaybeDefault<FieldDecoder<F1, StringDecoder>>,
            MaybeDefault<FieldDecoder<F2, StringDecoder>>,
            MaybeDefault<FieldDecoder<F3, Uint32Decoder>>,
            MaybeDefault<FieldDecoder<F4, StringDecoder>>,
        )>,
    >,
}
impl LocationDecoder {
    /// Makes a new `LocationDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Decode for LocationDecoder {
    type Item = Location;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.inner.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let (module, file, line, message) = track!(self.inner.finish_decoding())?;
        Ok(Location::new(module, file, line, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }
}
impl MessageDecode for LocationDecoder {}

/// Encoder for [Location].
///
/// See also [trackable.proto].
///
/// [Location]: https://docs.rs/trackable/0.2/trackable/struct.Location.html
/// [trackable.proto]: https://github.com/sile/protobuf_codec/blob/master/protobuf/trackable.proto
#[derive(Debug, Default)]
pub struct LocationEncoder {
    inner: MessageEncoder<
        Fields<(
            MaybeDefault<FieldEncoder<F1, StringEncoder>>,
            MaybeDefault<FieldEncoder<F2, StringEncoder>>,
            MaybeDefault<FieldEncoder<F3, Uint32Encoder>>,
            MaybeDefault<FieldEncoder<F4, StringEncoder>>,
        )>,
    >,
}
impl LocationEncoder {
    /// Makes a new `LocationEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Encode for LocationEncoder {
    type Item = Location;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.inner.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let item = (
            item.module_path().to_owned(),
            item.file().to_owned(),
            item.line(),
            item.message().to_owned(),
        );
        track!(self.inner.start_encoding(item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }
}
impl SizedEncode for LocationEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
    }
}
impl MessageEncode for LocationEncoder {}
