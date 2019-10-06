//! Encoders, decoders and traits for messages.
use crate::field::{FieldDecode, FieldEncode, UnknownFieldDecoder};
use crate::value::{ValueDecode, ValueEncode};
use crate::wire::{LengthDelimitedDecoder, LengthDelimitedEncoder, TagDecoder, WireType};
use bytecodec::combinator::{Map, MapErr, MapFrom, PreEncode, TryMap, TryMapFrom};
use bytecodec::{ByteCount, Decode, Encode, Eos, Error, ErrorKind, Result, SizedEncode};
use std;

/// This trait allows for decoding messages.
pub trait MessageDecode: Decode {}
impl<M, T, F> MessageDecode for Map<M, T, F>
where
    M: MessageDecode,
    F: Fn(M::Item) -> T,
{
}
impl<M, T, E, F> MessageDecode for TryMap<M, T, E, F>
where
    M: MessageDecode,
    F: Fn(M::Item) -> std::result::Result<T, E>,
    Error: From<E>,
{
}
impl<M, E, F> MessageDecode for MapErr<M, E, F>
where
    M: MessageDecode,
    F: Fn(Error) -> E,
    Error: From<E>,
{
}

/// This trait allows for encoding messages.
pub trait MessageEncode: Encode {}
impl<M: MessageEncode> MessageEncode for PreEncode<M> {}
impl<M, T, F> MessageEncode for MapFrom<M, T, F>
where
    M: MessageEncode,
    F: Fn(T) -> M::Item,
{
}
impl<M, T, E, F> MessageEncode for TryMapFrom<M, T, E, F>
where
    M: MessageEncode,
    F: Fn(T) -> std::result::Result<M::Item, E>,
    Error: From<E>,
{
}
impl<M, E, F> MessageEncode for MapErr<M, E, F>
where
    M: MessageEncode,
    F: Fn(Error) -> E,
    Error: From<E>,
{
}

/// Decoder for messages.
#[derive(Debug, Default)]
pub struct MessageDecoder<F> {
    tag: TagDecoder,
    field: F,
    unknown_field: UnknownFieldDecoder,
    started: bool,
    eos: bool, // end-of-stream
    target: DecodeTarget,
}
impl<F: FieldDecode> MessageDecoder<F> {
    /// Makes a new `MessageDecoder` instance.
    pub fn new(field_decoder: F) -> Self {
        MessageDecoder {
            tag: TagDecoder::default(),
            field: field_decoder,
            unknown_field: UnknownFieldDecoder::default(),
            started: false,
            eos: false,
            target: DecodeTarget::None,
        }
    }
}
impl<F: FieldDecode> Decode for MessageDecoder<F> {
    type Item = F::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        self.started = true;
        if self.eos {
            return Ok(0);
        }

        let mut offset = 0;
        while offset < buf.len() {
            match self.target {
                DecodeTarget::None | DecodeTarget::Tag => {
                    let size = track!(self.tag.decode(&buf[offset..], eos))?;
                    offset += size;
                    if size != 0 {
                        self.target = DecodeTarget::Tag;
                    }
                    if self.tag.is_idle() {
                        let tag = track!(self.tag.finish_decoding())?;
                        let started = track!(self.field.start_decoding(tag))?;
                        if started {
                            self.target = DecodeTarget::KnownField;
                        } else {
                            self.target = DecodeTarget::UnknownField;
                            track!(self.unknown_field.start_decoding(tag))?;
                        }
                    }
                }
                DecodeTarget::KnownField => {
                    bytecodec_try_decode!(self.field, offset, buf, eos);
                    self.target = DecodeTarget::None;
                }
                DecodeTarget::UnknownField => {
                    bytecodec_try_decode!(self.unknown_field, offset, buf, eos);
                    track!(self.unknown_field.finish_decoding())?;
                    self.target = DecodeTarget::None;
                }
            }
        }
        self.eos = eos.is_reached();
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(!self.started | self.eos, ErrorKind::IncompleteDecoding; self.target, self.started, self.eos);
        track_assert_eq!(
            self.target,
            DecodeTarget::None,
            ErrorKind::IncompleteDecoding
        );
        let v = track!(self.field.finish_decoding())?;
        self.started = false;
        self.eos = false;
        Ok(v)
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.eos {
            ByteCount::Finite(0)
        } else {
            match self.target {
                DecodeTarget::None => ByteCount::Unknown,
                DecodeTarget::Tag => self.tag.requiring_bytes(),
                DecodeTarget::KnownField => self.field.requiring_bytes(),
                DecodeTarget::UnknownField => self.unknown_field.requiring_bytes(),
            }
        }
    }

    fn is_idle(&self) -> bool {
        self.eos
    }
}
impl<F: FieldDecode> MessageDecode for MessageDecoder<F> {}

#[derive(Debug, PartialEq, Eq)]
enum DecodeTarget {
    None,
    Tag,
    KnownField,
    UnknownField,
}
impl Default for DecodeTarget {
    fn default() -> Self {
        DecodeTarget::None
    }
}

/// Decoder for embedded messages.
#[derive(Debug, Default)]
pub(crate) struct EmbeddedMessageDecoder<M>(LengthDelimitedDecoder<M>);
impl<M: MessageDecode> EmbeddedMessageDecoder<M> {
    /// Makes a new `EmbeddedMessageDecoder` instance.
    pub(crate) fn new(message_decoder: M) -> Self {
        EmbeddedMessageDecoder(LengthDelimitedDecoder::new(message_decoder))
    }
}
impl<M: MessageDecode> Decode for EmbeddedMessageDecoder<M> {
    type Item = M::Item;

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
impl<M: MessageDecode> ValueDecode for EmbeddedMessageDecoder<M> {
    fn wire_type(&self) -> WireType {
        WireType::LengthDelimited
    }
}

/// Encoder for messages.
#[derive(Debug, Default)]
pub struct MessageEncoder<F> {
    field: F,
}
impl<F: FieldEncode> MessageEncoder<F> {
    /// Makes a new `MessageEncoder` instance.
    pub fn new(field_encoder: F) -> Self {
        MessageEncoder {
            field: field_encoder,
        }
    }
}
impl<F: FieldEncode> Encode for MessageEncoder<F> {
    type Item = F::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.field.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.field.start_encoding(item))
    }

    fn is_idle(&self) -> bool {
        self.field.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.field.requiring_bytes()
    }
}
impl<F: FieldEncode + SizedEncode> SizedEncode for MessageEncoder<F> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.field.exact_requiring_bytes()
    }
}
impl<F: FieldEncode> MessageEncode for MessageEncoder<F> {}

/// Encoder for embedded messages.
#[derive(Debug, Default)]
pub(crate) struct EmbeddedMessageEncoder<M> {
    message: LengthDelimitedEncoder<M>,
}
impl<M: MessageEncode + SizedEncode> EmbeddedMessageEncoder<M> {
    /// Makes a new `EmbeddedMessageEncoder` instance.
    pub(crate) fn new(message_encoder: M) -> Self {
        EmbeddedMessageEncoder {
            message: LengthDelimitedEncoder::new(message_encoder),
        }
    }
}
impl<M: MessageEncode + SizedEncode> Encode for EmbeddedMessageEncoder<M> {
    type Item = M::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.message.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.message.start_encoding(item))
    }

    fn is_idle(&self) -> bool {
        self.message.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.message.requiring_bytes()
    }
}
impl<M: MessageEncode + SizedEncode> SizedEncode for EmbeddedMessageEncoder<M> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.message.exact_requiring_bytes()
    }
}
impl<M: MessageEncode + SizedEncode> ValueEncode for EmbeddedMessageEncoder<M> {
    fn wire_type(&self) -> WireType {
        WireType::LengthDelimited
    }
}
