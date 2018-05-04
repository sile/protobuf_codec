use wire::WireType;

pub trait Value: Default {
    fn wire_type(&self) -> WireType;
}
