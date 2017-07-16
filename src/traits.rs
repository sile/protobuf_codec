use wire::WireType;

pub trait Tag {
    fn number() -> u32;
}

pub trait Pattern {
    type Value: Default;
}
impl Pattern for Vec<u8> {
    type Value = Vec<u8>;
}

pub trait Type: Pattern {
    fn wire_type() -> WireType;
}

pub trait Field: Pattern {}

pub trait SingularField: Field {}

// pub trait DerivedType: Sized {
//     type BaseType: Type;
//     type BaseValue;
//     fn from_base(value: Self::BaseValue) -> Result<Self>;
//     fn into_base(self) -> Self::BaseValue;
//     // fn encode<W: Write>(self, writer: W) -> <Self::BaseType as Encode<W>>::Future
//     // where
//     //     Self::BaseType: Encode<W, Value = Self::BaseValue>,
//     // {
//     //     Self::BaseType::encode(writer, self.into_base())
//     // }
//     // fn decode<R: Read>(reader: R) -> <Self::BaseType as Decode<R>>::Future
//     // where
//     //     Self::BaseType: Decode<R, Value = Self::BaseValue>,
//     // {
//     //     Self::BaseType::decode(reader)
//     // }
// }
