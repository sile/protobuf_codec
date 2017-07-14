use wire::WireType;

pub trait Tag {
    fn number() -> u32;
}

pub trait Type {
    fn wire_type() -> WireType;
}
pub trait Field {}
pub trait SingularField: Field {}
