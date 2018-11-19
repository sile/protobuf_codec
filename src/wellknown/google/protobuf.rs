//! `package google.protobuf;`
//!
//! See also [google/protobuf].
//!
//! [google/protobuf]: https://github.com/google/protobuf/tree/master/src/google/protobuf
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, Result, SizedEncode};
use std::time::Duration;

use field::num::{F1, F2};
use field::{FieldDecoder, FieldEncoder, Fields, MaybeDefault};
use message::{MessageDecode, MessageDecoder, MessageEncode, MessageEncoder};
use scalar::{Int32Decoder, Int32Encoder, Int64Decoder, Int64Encoder};

/// Decoder for [Empty] Message.
///
/// [Empty]: https://github.com/google/protobuf/blob/master/src/google/protobuf/empty.proto
#[derive(Debug, Default)]
pub struct EmptyMessageDecoder(MessageDecoder<Fields<()>>);
impl EmptyMessageDecoder {
    /// Makes a new `EmptyMessageDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Decode for EmptyMessageDecoder {
    type Item = ();

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.0.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track!(self.0.finish_decoding())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }
}
impl MessageDecode for EmptyMessageDecoder {}

/// Encoder for [Empty] Message.
///
/// [Empty]: https://github.com/google/protobuf/blob/master/src/google/protobuf/empty.proto
#[derive(Debug, Default)]
pub struct EmptyMessageEncoder(MessageEncoder<Fields<()>>);
impl EmptyMessageEncoder {
    /// Makes a new `EmptyMessageEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Encode for EmptyMessageEncoder {
    type Item = ();

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, (): Self::Item) -> Result<()> {
        track!(self.0.start_encoding(()))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }
}
impl SizedEncode for EmptyMessageEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}
impl MessageEncode for EmptyMessageEncoder {}

/// [Duration] message.
///
/// [Duration]: https://github.com/google/protobuf/blob/master/src/google/protobuf/duration.proto
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DurationMessage {
    seconds: i64,
    nanos: i32,
}
impl DurationMessage {
    /// Minimum value of the seconds part of `DurationMessage`.
    pub const MIN_SECONDS: i64 = -315_576_000_000;

    /// Maximum value of the seconds part of `DurationMessage`.
    pub const MAX_SECONDS: i64 = 315_576_000_000;

    /// Minimum value of the nanoseconds part of `DurationMessage`.
    pub const MIN_NANOS: i32 = -999_999_999;

    /// Maximum value of the nanoseconds part of `DurationMessage`.
    pub const MAX_NANOS: i32 = 999_999_999;

    /// Makes a new `DurationMessage` instance.
    ///
    /// # Errors
    ///
    /// If `seconds` or `nanos` are too small or too large,
    /// an `ErrorKind::InvalidInput` error will be returned.
    pub fn new(seconds: i64, nanos: i32) -> Result<Self> {
        track_assert!(Self::MIN_SECONDS <= seconds, ErrorKind::InvalidInput; seconds, nanos);
        track_assert!(seconds <= Self::MAX_SECONDS, ErrorKind::InvalidInput; seconds, nanos);
        track_assert!(Self::MIN_NANOS <= nanos, ErrorKind::InvalidInput; seconds, nanos);
        track_assert!(nanos <= Self::MAX_NANOS, ErrorKind::InvalidInput; seconds, nanos);
        Ok(DurationMessage { seconds, nanos })
    }

    /// Makes a new `DurationMessage` instance from `Duration`.
    ///
    /// # Errors
    ///
    /// If `d` has a too large value, an `ErrorKind::InvalidInput` error will be returned.
    pub fn from_duration(d: Duration) -> Result<Self> {
        track!(Self::new(d.as_secs() as i64, d.subsec_nanos() as i32))
    }

    /// Returns the seconds part of the instance.
    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    /// Returns the nanoseconds part of the instance.
    pub fn nanos(&self) -> i32 {
        self.nanos
    }

    /// Converts `DurationMessage` to `std::time::Duration`.
    ///
    /// If the instance has a minus field, this method will return `None`.
    pub fn to_duration(&self) -> Option<Duration> {
        if self.seconds >= 0 && self.nanos >= 0 {
            Some(Duration::new(self.seconds as u64, self.nanos as u32))
        } else {
            None
        }
    }
}

/// Decoder for [Duration] message.
///
/// [Duration]: https://github.com/google/protobuf/blob/master/src/google/protobuf/duration.proto
#[derive(Debug, Default)]
pub struct DurationMessageDecoder {
    inner: MessageDecoder<
        Fields<(
            MaybeDefault<FieldDecoder<F1, Int64Decoder>>,
            MaybeDefault<FieldDecoder<F2, Int32Decoder>>,
        )>,
    >,
}
impl DurationMessageDecoder {
    /// Makes a new `DurationMessageDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Decode for DurationMessageDecoder {
    type Item = DurationMessage;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.inner.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let (seconds, nanos) = track!(self.inner.finish_decoding())?;
        track!(DurationMessage::new(seconds, nanos))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }
}
impl MessageDecode for DurationMessageDecoder {}

/// Encoder for [Duration] message.
///
/// [Duration]: https://github.com/google/protobuf/blob/master/src/google/protobuf/duration.proto
#[derive(Debug, Default)]
pub struct DurationMessageEncoder {
    inner: MessageEncoder<
        Fields<(
            MaybeDefault<FieldEncoder<F1, Int64Encoder>>,
            MaybeDefault<FieldEncoder<F2, Int32Encoder>>,
        )>,
    >,
}
impl DurationMessageEncoder {
    /// Makes a new `DurationMessageEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Encode for DurationMessageEncoder {
    type Item = DurationMessage;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.inner.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let item = (item.seconds, item.nanos);
        track!(self.inner.start_encoding(item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }
}
impl SizedEncode for DurationMessageEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
    }
}
impl MessageEncode for DurationMessageEncoder {}

/// Decoder for `std::time::Duration`.
///
/// This is based on [DurationMessageDecoder](./struct.DurationMessageDecoder.html).
#[derive(Debug, Default)]
pub struct StdDurationDecoder(DurationMessageDecoder);
impl StdDurationDecoder {
    /// Makes a new `StdDurationDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Decode for StdDurationDecoder {
    type Item = Duration;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.0.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let m = track!(self.0.finish_decoding())?;
        let d = track_assert_some!(m.to_duration(), ErrorKind::InvalidInput; m);
        Ok(d)
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }
}
impl MessageDecode for StdDurationDecoder {}

/// Encoder for `std::time::Duration`.
///
/// This is based on [DurationMessageEncoder](./struct.DurationMessageEncoder.html).
#[derive(Debug, Default)]
pub struct StdDurationEncoder(DurationMessageEncoder);
impl StdDurationEncoder {
    /// Makes a new `StdDurationEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Encode for StdDurationEncoder {
    type Item = Duration;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let item = track!(DurationMessage::from_duration(item))?;
        track!(self.0.start_encoding(item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }
}
impl SizedEncode for StdDurationEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}
impl MessageEncode for StdDurationEncoder {}

#[cfg(test)]
mod tests {
    use bytecodec::DecodeExt;

    use super::*;
    use field::{num::F2, MessageFieldDecoder};
    use message::MessageDecoder;

    #[test]
    fn duration_decoder_works() {
        let mut decoder: MessageDecoder<MessageFieldDecoder<F2, StdDurationDecoder>> =
            Default::default();

        let input = [18, 0];
        track_try_unwrap!(decoder.decode_from_bytes(&input[..]));
    }
}
