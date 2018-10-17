use bytecodec::combinator::{Map, MapErr, MapFrom, TryMap, TryMapFrom};
use bytecodec::{Decode, Encode, Error, SizedEncode};
use std;

use wire::WireType;

/// This trait allows for decoding field values.
///
/// `Decode::Item` is the type of the field.
pub trait ValueDecode: Decode {
    /// Returns the wire type of the value.
    fn wire_type(&self) -> WireType;
}
impl<V, T, F> ValueDecode for Map<V, T, F>
where
    V: ValueDecode,
    F: Fn(V::Item) -> T,
{
    fn wire_type(&self) -> WireType {
        self.inner_ref().wire_type()
    }
}
impl<V, T, E, F> ValueDecode for TryMap<V, T, E, F>
where
    V: ValueDecode,
    F: Fn(V::Item) -> std::result::Result<T, E>,
    Error: From<E>,
{
    fn wire_type(&self) -> WireType {
        self.inner_ref().wire_type()
    }
}
impl<V, E, F> ValueDecode for MapErr<V, E, F>
where
    V: ValueDecode,
    F: Fn(Error) -> E,
    Error: From<E>,
{
    fn wire_type(&self) -> WireType {
        self.inner_ref().wire_type()
    }
}

/// This trait allows for encoding field values.
///
/// `Encode::Item` is the type of the field.
pub trait ValueEncode: Encode {
    /// Returns the wire type of the value.
    fn wire_type(&self) -> WireType;
}
impl<V, T, F> ValueEncode for MapFrom<V, T, F>
where
    V: ValueEncode,
    F: Fn(T) -> V::Item,
{
    fn wire_type(&self) -> WireType {
        self.inner_ref().wire_type()
    }
}
impl<V, T, E, F> ValueEncode for TryMapFrom<V, T, E, F>
where
    V: ValueEncode,
    F: Fn(T) -> std::result::Result<V::Item, E>,
    Error: From<E>,
{
    fn wire_type(&self) -> WireType {
        self.inner_ref().wire_type()
    }
}
impl<V, E, F> ValueEncode for MapErr<V, E, F>
where
    V: ValueEncode,
    F: Fn(Error) -> E,
    Error: From<E>,
{
    fn wire_type(&self) -> WireType {
        self.inner_ref().wire_type()
    }
}

/// This trait allows for decoding values of map key fields.
pub trait MapKeyDecode: ValueDecode {}
impl<V, T, F> MapKeyDecode for Map<V, T, F>
where
    V: MapKeyDecode,
    F: Fn(V::Item) -> T,
{
}
impl<V, T, E, F> MapKeyDecode for TryMap<V, T, E, F>
where
    V: MapKeyDecode,
    F: Fn(V::Item) -> std::result::Result<T, E>,
    Error: From<E>,
{}
impl<V, E, F> MapKeyDecode for MapErr<V, E, F>
where
    V: MapKeyDecode,
    F: Fn(Error) -> E,
    Error: From<E>,
{}

/// This trait allows for encoding values of map key fields.
pub trait MapKeyEncode: ValueEncode {}
impl<V, T, F> MapKeyEncode for MapFrom<V, T, F>
where
    V: MapKeyEncode,
    F: Fn(T) -> V::Item,
{
}
impl<V, T, E, F> MapKeyEncode for TryMapFrom<V, T, E, F>
where
    V: MapKeyEncode,
    F: Fn(T) -> std::result::Result<V::Item, E>,
    Error: From<E>,
{}
impl<V, E, F> MapKeyEncode for MapErr<V, E, F>
where
    V: MapKeyEncode,
    F: Fn(Error) -> E,
    Error: From<E>,
{}

/// This trait allows for decoding numeric values.
pub trait NumericValueDecode: ValueDecode {}
impl<V, T, F> NumericValueDecode for Map<V, T, F>
where
    V: NumericValueDecode,
    F: Fn(V::Item) -> T,
{}
impl<V, T, E, F> NumericValueDecode for TryMap<V, T, E, F>
where
    V: NumericValueDecode,
    F: Fn(V::Item) -> std::result::Result<T, E>,
    Error: From<E>,
{}
impl<V, E, F> NumericValueDecode for MapErr<V, E, F>
where
    V: NumericValueDecode,
    F: Fn(Error) -> E,
    Error: From<E>,
{}

/// This trait allows for encoding numeric values.
pub trait NumericValueEncode: ValueEncode + SizedEncode {}
impl<V, T, F> NumericValueEncode for MapFrom<V, T, F>
where
    V: NumericValueEncode,
    F: Fn(T) -> V::Item,
{}
impl<V, T, E, F> NumericValueEncode for TryMapFrom<V, T, E, F>
where
    V: NumericValueEncode,
    F: Fn(T) -> std::result::Result<V::Item, E>,
    Error: From<E>,
{}
impl<V, E, F> NumericValueEncode for MapErr<V, E, F>
where
    V: NumericValueEncode,
    F: Fn(Error) -> E,
    Error: From<E>,
{}
