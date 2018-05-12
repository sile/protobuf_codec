use bytecodec::{Decode, Encode, ExactBytesEncode, Result};

use wire::WireType;

/// This trait allows for decoding field values.
///
/// `Decode::Item` is the type of the field.
pub trait ValueDecode: Decode {
    /// Returns the wire type of the value.
    fn wire_type(&self) -> WireType;

    /// Merges duplicate values.
    fn merge_values(old: &mut Self::Item, new: Self::Item);
}

/// This trait allows for decoding optional field values.
pub trait OptionalValueDecode: ValueDecode {
    /// The type of the optional field.
    type Optional: Default + From<Self::Item>;

    /// Merges duplicate optional values.
    fn merge_optional_values(old: &mut Self::Optional, new: Self::Optional);
}

/// This trait allows for encoding field values.
///
/// `Encode::Item` is the type of the field.
pub trait ValueEncode: Encode {
    /// Returns the wire type of the value.
    fn wire_type(&self) -> WireType;
}

/// This trait allows for encoding optional field values.
pub trait OptionalValueEncode: ValueEncode {
    /// The type of the optional field.
    type Optional;

    /// Starts encoding the given field value if needed.
    ///
    /// If it is the default value of the field, the encoder will omit encoding the value.
    fn start_encoding_if_needed(&mut self, item: Self::Optional) -> Result<()>;
}

/// This trait allows for decoding values of map key fields.
pub trait MapKeyDecode: ValueDecode {}

/// This trait allows for encoding values of map key fields.
pub trait MapKeyEncode: ValueEncode {}

/// This trait allows for decoding numeric values.
pub trait NumericValueDecode: ValueDecode {}

/// This trait allows for encoding numeric values.
pub trait NumericValueEncode: ValueEncode + ExactBytesEncode {}
