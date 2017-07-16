use std::marker::PhantomData;

use traits::{self, Tag, Type, SingularField, Pattern};
use variants::Variant2;

pub struct Field<T: Tag, F: Type>(PhantomData<(T, F)>);
impl<T: Tag, F: Type> Pattern for Field<T, F> {
    type Value = F::Value;
}
impl<T: Tag, F: Type> traits::Field for Field<T, F> {}
impl<T: Tag, F: Type> traits::SingularField for Field<T, F> {}

pub struct RepeatedField<T: Tag, F: Type>(PhantomData<(T, F)>);
impl<T: Tag, F: Type> Pattern for RepeatedField<T, F> {
    type Value = Vec<F::Value>;
}
impl<T: Tag, F: Type> traits::Field for RepeatedField<T, F> {}

pub struct PackedRepeatedField<T: Tag, F: Type>(PhantomData<(T, F)>);
impl<T: Tag, F: Type> Pattern for PackedRepeatedField<T, F> {
    type Value = Vec<F::Value>;
}
impl<T: Tag, F: Type> traits::Field for PackedRepeatedField<T, F> {}

pub struct Oneof<Fields>(PhantomData<Fields>);
impl<A, B> Pattern for Oneof<(A, B)>
where
    A: SingularField,
    B: SingularField,
{
    type Value = Option<Variant2<A::Value, B::Value>>;
}
impl<A, B> traits::Field for Oneof<(A, B)>
where
    A: SingularField,
    B: SingularField,
{
}
