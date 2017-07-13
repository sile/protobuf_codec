use std::collections::HashMap;
use std::hash::Hash;

use {Type, Field, Payload, WireType};

#[derive(Debug, Clone, Copy)]
pub struct Enum;
impl Type for Enum {
    type Value = u64;
    fn wire_type() -> WireType {
        WireType::Varint
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Packed<T>(pub T);
impl<T: Type> Type for Packed<T> {
    type Value = Vec<T::Value>;
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Map<K, V> {
    pub key: K,
    pub value: V,
}
impl<K: Type, V: Type> Type for Map<K, V>
where
    K::Value: Eq + Hash,
{
    type Value = HashMap<K::Value, V::Value>;
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Message<T> {
    pub name: &'static str,
    pub fields: T,
}
impl<A: Field> Type for Message<(A,)> {
    type Value = (A::Value,);
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}
impl<A: Field, B: Field> Type for Message<(A, B)> {
    type Value = (A::Value, B::Value);
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}
impl<A: Field, B: Field, C: Field> Type for Message<(A, B, C)> {
    type Value = (A::Value, B::Value, C::Value);
    fn wire_type() -> WireType {
        WireType::LengthDelimited
    }
}
impl<A: Field, B: Field> Payload for Message<(A, B)> {
    type Value = (A::Value, B::Value);
}
