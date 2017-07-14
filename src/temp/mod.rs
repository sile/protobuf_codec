pub trait Tag {
    fn number() -> u32;
}
pub trait Type {}
pub trait Field {}
pub trait SingularField: Field {}

pub mod fields {
    pub struct Field<T, F> {
        pub tag: T,
        pub ty: F,
    }
    impl<T, F> super::Field for Field<T, F> {}
    impl<T, F> super::SingularField for Field<T, F> {}

    pub struct RepeatedField<T, F> {
        pub tag: T,
        pub ty: F,
    }
    impl<T, F> super::Field for RepeatedField<T, F> {}

    pub struct PackedRepeatedField<T, F> {
        pub tag: T,
        pub ty: F,
    }
    impl<T, F> super::Field for PackedRepeatedField<T, F> {}

    pub struct Oneof<T> {
        pub fields: T,
    }

    pub type Packed<T, F> = PackedRepeatedField<T, F>;
    pub type Repeated<T, F> = RepeatedField<T, F>;
}

pub mod types {}

pub struct Message<T> {
    pub fields: T,
}
