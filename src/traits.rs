use Result;
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

pub trait DerivedType: Sized + Default {
    type Base: Type;
    fn from_base(value: <Self::Base as Pattern>::Value) -> Result<Self>;
    fn into_base(self) -> <Self::Base as Pattern>::Value;
    // fn encode<W: Write>(self, writer: W) -> <Self::Base as Encode<W>>::Future
    // where
    //     Self::Base: Encode<W, Value = Self::BaseValue>,
    // {
    //     Self::Base::encode(writer, self.into_base())
    // }
    // fn decode<R: Read>(reader: R) -> <Self::Base as Decode<R>>::Future
    // where
    //     Self::Base: Decode<R, Value = Self::BaseValue>,
    // {
    //     Self::Base::decode(reader)
    // }
}
