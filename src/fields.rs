use traits::{self, Tag, Type, SingularField};
use variants;

#[derive(Debug, Default)]
pub struct Field<T: Tag, V: Type> {
    pub tag: T,
    pub value: V,
}
impl<T: Tag, V: Type> traits::Field for Field<T, V> {}
impl<T: Tag, V: Type> traits::SingularField for Field<T, V> {}
impl<T: Tag, V: Type> From<V> for Field<T, V> {
    fn from(f: V) -> Self {
        Field {
            tag: T::default(),
            value: f,
        }
    }
}

#[derive(Debug, Default)]
pub struct RepeatedField<T: Tag, V: Type> {
    pub tag: T,
    pub values: Vec<V>,
}
impl<T: Tag, V: Type> traits::Field for RepeatedField<T, V> {}
impl<T: Tag, V: Type> From<Vec<V>> for RepeatedField<T, V> {
    fn from(f: Vec<V>) -> Self {
        RepeatedField {
            tag: T::default(),
            values: f,
        }
    }
}

#[derive(Debug, Default)]
pub struct PackedRepeatedField<T: Tag, V: Type> {
    pub tag: T,
    pub values: Vec<V>,
}
impl<T: Tag, V: Type> traits::Field for PackedRepeatedField<T, V> {}
impl<T: Tag, V: Type> From<Vec<V>> for PackedRepeatedField<T, V> {
    fn from(f: Vec<V>) -> Self {
        PackedRepeatedField {
            tag: T::default(),
            values: f,
        }
    }
}

#[derive(Debug, Default)]
pub struct Oneof1<A>
where
    A: SingularField,
{
    pub field: Option<A>,
}
impl<A> traits::Field for Oneof1<A>
where
    A: SingularField,
{
}
impl<A> From<Option<A>> for Oneof1<A>
where
    A: SingularField,
{
    fn from(f: Option<A>) -> Self {
        Oneof1 { field: f }
    }
}

#[derive(Debug, Default)]
pub struct Oneof2<A, B>
where
    A: SingularField,
    B: SingularField,
{
    pub field: Option<variants::Variant2<A, B>>,
}
impl<A, B> traits::Field for Oneof2<A, B>
where
    A: SingularField,
    B: SingularField,
{
}
impl<A, B> From<Option<variants::Variant2<A, B>>> for Oneof2<A, B>
where
    A: SingularField,
    B: SingularField,
{
    fn from(f: Option<variants::Variant2<A, B>>) -> Self {
        Oneof2 { field: f }
    }
}
