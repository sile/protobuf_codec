use traits::{self, FieldType, Map, SingularField, Tag};
use variants;

#[derive(Debug, Default)]
pub struct Field<T: Tag, V: FieldType> {
    pub tag: T,
    pub value: V,
}
impl<T: Tag, V: FieldType> traits::Field for Field<T, V> {}
impl<T: Tag, V: FieldType> traits::SingularField for Field<T, V> {}
impl<T: Tag, V: FieldType> From<V> for Field<T, V> {
    fn from(f: V) -> Self {
        Field {
            tag: T::default(),
            value: f,
        }
    }
}

#[derive(Debug, Default)]
pub struct RepeatedField<T: Tag, V: FieldType> {
    pub tag: T,
    pub values: Vec<V>,
}
impl<T: Tag, V: FieldType> traits::Field for RepeatedField<T, V> {}
impl<T: Tag, V: FieldType> From<Vec<V>> for RepeatedField<T, V> {
    fn from(f: Vec<V>) -> Self {
        RepeatedField {
            tag: T::default(),
            values: f,
        }
    }
}

#[derive(Debug, Default)]
pub struct PackedRepeatedField<T: Tag, V: FieldType> {
    pub tag: T,
    pub values: Vec<V>,
}
impl<T: Tag, V: FieldType> traits::Field for PackedRepeatedField<T, V> {}
impl<T: Tag, V: FieldType> From<Vec<V>> for PackedRepeatedField<T, V> {
    fn from(f: Vec<V>) -> Self {
        PackedRepeatedField {
            tag: T::default(),
            values: f,
        }
    }
}

#[derive(Debug, Default)]
pub struct MapField<T: Tag, M: Map> {
    pub tag: T,
    pub map: M,
}
impl<T: Tag, M: Map> traits::Field for MapField<T, M> {}

macro_rules! define_oneof {
    ($name:ident, $variant:ident, $($param:ident),*) => {
        #[derive(Debug)]
        pub struct $name<$($param),*> {
            pub field: Option<variants::$variant<$($param),*>>,
        }
        impl<$($param),*> traits::Field for $name<$($param),*>
        where
            $($param: SingularField),*
        {
        }
        impl<$($param),*> From<variants::$variant<$($param),*>> for $name<$($param),*> {
            fn from(f: variants::$variant<$($param),*>) -> Self {
                $name { field: Some(f) }
            }
        }
        impl<$($param),*> From<Option<variants::$variant<$($param),*>>> for $name<$($param),*> {
            fn from(f: Option<variants::$variant<$($param),*>>) -> Self {
                $name { field: f }
            }
        }
        impl<$($param),*> Default for $name<$($param),*> {
            fn default() -> Self {
                $name { field: None }
            }
        }
    }
}

define_oneof!(Oneof1, Variant1, A);
define_oneof!(Oneof2, Variant2, A, B);
define_oneof!(Oneof3, Variant3, A, B, C);
define_oneof!(Oneof4, Variant4, A, B, C, D);
define_oneof!(Oneof5, Variant5, A, B, C, D, E);
define_oneof!(Oneof6, Variant6, A, B, C, D, E, F);
define_oneof!(Oneof7, Variant7, A, B, C, D, E, F, G);
define_oneof!(Oneof8, Variant8, A, B, C, D, E, F, G, H);
