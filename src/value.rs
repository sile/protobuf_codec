use wire::WireType;

// TODO: WireValue
pub trait Value: Default {
    fn wire_type(&self) -> WireType;
}
