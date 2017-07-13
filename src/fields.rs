use {Tag, Type, Field};
use composites::Packed;
use variants::{Variant2, Variant3};

#[derive(Debug, Clone, Copy)]
pub struct Singular<T> {
    pub tag: Tag,
    pub name: &'static str,
    pub value: T,
}
impl<T: Type> Field for Singular<T> {
    type Value = T::Value;
}

#[derive(Debug, Clone, Copy)]
pub struct Repeated<T> {
    pub tag: Tag,
    pub name: &'static str,
    pub value: T,
}
impl<T: Type> Field for Repeated<T> {
    type Value = Vec<T::Value>;
}

#[derive(Debug, Clone, Copy)]
pub struct PackedRepeated<T> {
    pub tag: Tag,
    pub name: &'static str,
    pub value: Packed<T>,
}
impl<T: Type> Field for PackedRepeated<T> {
    type Value = Vec<T::Value>;
}

#[derive(Debug, Clone, Copy)]
pub struct Ignore;
impl Field for Ignore {
    type Value = ();
}

#[derive(Debug, Clone, Copy)]
pub struct ReservedTag(pub Tag);
impl Field for ReservedTag {
    type Value = ();
}

#[derive(Debug, Clone, Copy)]
pub struct ReservedName(&'static str);
impl Field for ReservedName {
    type Value = ();
}

#[derive(Debug, Clone, Copy)]
pub struct Oneof<T> {
    pub name: &'static str,
    pub fields: T,
}
impl<A: Field, B: Field> Field for Oneof<(A, B)> {
    type Value = Variant2<A::Value, B::Value>;
}
impl<A: Field, B: Field, C: Field> Field for Oneof<(A, B, C)> {
    type Value = Variant3<A::Value, B::Value, C::Value>;
}
