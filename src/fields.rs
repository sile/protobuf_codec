use std::marker::PhantomData;

use traits::{self, Tag, Type, SingularField};

pub struct Field<T: Tag, F: Type>(PhantomData<(T, F)>);
impl<T: Tag, F: Type> traits::Field for Field<T, F> {}
impl<T: Tag, F: Type> traits::SingularField for Field<T, F> {}

pub struct RepeatedField<T: Tag, F: Type>(PhantomData<(T, F)>);
impl<T: Tag, F: Type> traits::Field for RepeatedField<T, F> {}

pub struct PackedRepeatedField<T: Tag, F: Type>(PhantomData<(T, F)>);
impl<T: Tag, F: Type> traits::Field for PackedRepeatedField<T, F> {}

pub struct Oneof<Fields>(PhantomData<Fields>);
impl<A, B> traits::Field for Oneof<(A, B)>
where
    A: SingularField,
    B: SingularField,
{
}
