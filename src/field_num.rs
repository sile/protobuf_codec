use bytecodec::{ErrorKind, Result};

/// Field number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldNum(u32);
impl FieldNum {
    /// Makes a new `FieldNum` instance.
    ///
    /// # Errors
    ///
    /// [The language guide] says about the valid values of a field number as follows:
    ///
    /// > The smallest field number you can specify is `1`, and the largest is `2^29 - 1`, or `536,870,911`.
    /// > You also cannot use the numbers `19000` through `19999`, as they are reserved for
    /// > the Protocol Buffers implementation
    ///
    /// If `n` violates this restriction, an `ErrorKind::InvalidInput` error will be returned.
    ///
    /// [the language guide]: https://developers.google.com/protocol-buffers/docs/proto3
    pub fn new(n: u32) -> Result<Self> {
        track_assert_ne!(n, 0, ErrorKind::InvalidInput);
        track_assert!(n < (1 << 29), ErrorKind::InvalidInput; n);
        track_assert!(!(19_000 <= n && n < 20_000), ErrorKind::InvalidInput; n);
        Ok(FieldNum(n))
    }

    /// Makes a new `FieldNum` instance without checking the value.
    pub unsafe fn new_unchecked(n: u32) -> Self {
        FieldNum(n)
    }

    /// Returns the value of the field number.
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

macro_rules! impl_from {
    ($ty:ty, $n:expr) => {
        impl From<$ty> for FieldNum {
            fn from(_: $ty) -> Self {
                FieldNum($n)
            }
        }
    };
}

/// Field number `1`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F1;
impl_from!(F1, 1);

/// Field number `2`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F2;
impl_from!(F2, 2);

/// Field number `3`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F3;
impl_from!(F3, 3);

/// Field number `4`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F4;
impl_from!(F4, 4);

/// Field number `5`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F5;
impl_from!(F5, 5);

/// Field number `6`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F6;
impl_from!(F6, 6);

/// Field number `7`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F7;
impl_from!(F7, 7);

/// Field number `8`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F8;
impl_from!(F8, 8);

/// Field number `9`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F9;
impl_from!(F9, 9);

/// Field number `10`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F10;
impl_from!(F10, 10);

/// Field number `11`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F11;
impl_from!(F11, 11);

/// Field number `12`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F12;
impl_from!(F12, 12);

/// Field number `13`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F13;
impl_from!(F13, 13);

/// Field number `14`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F14;
impl_from!(F14, 14);

/// Field number `15`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F15;
impl_from!(F15, 15);

/// Field number `16`.
#[derive(Debug, Default, Clone, Copy)]
pub struct F16;
impl_from!(F16, 16);
