use std::io::{Read, Take};
use trackable::error::ErrorKindExt;

use {Tag, WireType, ErrorKind};
use decode::{Decode, DecodeError, DecodeField, DecodeFieldResult};
use decode::futures::{self, Unused, Push, DecodeVariant2, Ignore};
use fields::{self, Singular};

macro_rules! check_tag {
    ($reader:expr, $acc:expr, $actual:expr, $expected:expr) => {
        if $actual != $expected {
            return DecodeFieldResult::NotTarget($reader, $acc);
        }
    }
}
macro_rules! assert_wire_type {
    ($reader:expr, $actual:expr, $expected:expr) => {
        if $actual != $expected {
            let cause = format!("Unexpected wire type: actual={:?}, expected={:?}",
                                $actual, $expected);
            let error = track!(ErrorKind::Invalid.cause(cause)).into();
            return DecodeFieldResult::Err(DecodeError::new($reader, error));
        }
    }
}
macro_rules! try_decode_variant {
    ($field:expr, $reader:expr, $tag:expr, $wire_type:expr, $variant:expr) => {
        match $field.decode_field($reader, $tag, $wire_type, Default::default()) {
            DecodeFieldResult::Err(e) => return DecodeFieldResult::Err(track!(e)),
            DecodeFieldResult::Ok(v) => return DecodeFieldResult::Ok($variant(v)),
            DecodeFieldResult::NotTarget(r, _) => r
        }
    }
}

impl<R: Read, T: Decode<R>> DecodeField<R> for Singular<T> {
    type Future = T::Future;
    fn decode_field(
        self,
        reader: R,
        tag: Tag,
        wire_type: WireType,
        acc: Self::Value,
    ) -> DecodeFieldResult<Self::Future, R, Self::Value> {
        check_tag!(reader, acc, tag, self.tag);
        assert_wire_type!(reader, wire_type, T::wire_type());
        DecodeFieldResult::Ok(self.value.decode(reader))
    }
}

impl<R: Read> DecodeField<R> for fields::Ignore {
    type Future = Ignore<R>;
    fn decode_field(
        self,
        reader: R,
        _tag: Tag,
        wire_type: WireType,
        _acc: Self::Value,
    ) -> DecodeFieldResult<Self::Future, R, Self::Value> {
        DecodeFieldResult::Ok(Ignore::new(reader, wire_type))
    }
}

impl<R: Read> DecodeField<R> for fields::ReservedTag {
    type Future = Unused<R, ()>;
    fn decode_field(
        self,
        reader: R,
        _tag: Tag,
        _wire_type: WireType,
        acc: Self::Value,
    ) -> DecodeFieldResult<Self::Future, R, Self::Value> {
        DecodeFieldResult::NotTarget(reader, acc)
    }
}

impl<R: Read> DecodeField<R> for fields::ReservedName {
    type Future = Unused<R, ()>;
    fn decode_field(
        self,
        reader: R,
        _tag: Tag,
        _wire_type: WireType,
        acc: Self::Value,
    ) -> DecodeFieldResult<Self::Future, R, Self::Value> {
        DecodeFieldResult::NotTarget(reader, acc)
    }
}

impl<R: Read, T: Decode<R>> DecodeField<R> for fields::Repeated<T> {
    type Future = Push<R, T::Future, T::Value>;
    fn decode_field(
        self,
        reader: R,
        tag: Tag,
        wire_type: WireType,
        acc: Self::Value,
    ) -> DecodeFieldResult<Self::Future, R, Self::Value> {
        check_tag!(reader, acc, tag, self.tag);
        assert_wire_type!(reader, wire_type, T::wire_type());
        DecodeFieldResult::Ok(Push::new(self.value.decode(reader), acc))
    }
}

impl<R: Read, T> DecodeField<R> for fields::PackedRepeated<T>
where
    T: Decode<Take<R>> + Clone,
{
    type Future = futures::DecodePacked<R, T>;
    fn decode_field(
        self,
        reader: R,
        tag: Tag,
        wire_type: WireType,
        acc: Self::Value,
    ) -> DecodeFieldResult<Self::Future, R, Self::Value> {
        check_tag!(reader, acc, tag, self.tag);
        assert_wire_type!(reader, wire_type, WireType::LengthDelimited);
        DecodeFieldResult::Ok(self.value.decode(reader))
    }
}

impl<R, A, B> DecodeField<R> for fields::Oneof<(A, B)>
where
    R: Read,
    A: DecodeField<R>,
    B: DecodeField<R>,
{
    type Future = DecodeVariant2<R, A, B>;
    fn decode_field(
        self,
        reader: R,
        tag: Tag,
        wire_type: WireType,
        acc: Self::Value,
    ) -> DecodeFieldResult<Self::Future, R, Self::Value> {
        let r = reader;
        let r = try_decode_variant!(self.fields.0, r, tag, wire_type, DecodeVariant2::A);
        let r = try_decode_variant!(self.fields.1, r, tag, wire_type, DecodeVariant2::B);
        DecodeFieldResult::NotTarget(r, acc)
    }
}
