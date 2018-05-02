use super::wire::WireType;

pub trait Value {
    fn wire_type(&self) -> WireType;
}
