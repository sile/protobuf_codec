//! Field tag.
use bytecodec::{ErrorKind, Result};

/// Field tag (a.k.a, field number).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tag(u32);
impl Tag {
    /// Makes a new `Tag` instance.
    ///
    /// # Errors
    ///
    /// [The language guide] says about the valid values of a tag as follows:
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
        track_assert!(n < (2 << 29), ErrorKind::InvalidInput; n);
        track_assert!(!(19_000 <= n && n < 20_000), ErrorKind::InvalidInput; n);
        Ok(Tag(n))
    }

    /// Makes a new `Tag` instance without checking the value.
    pub unsafe fn new_unchecked(n: u32) -> Self {
        Tag(n)
    }

    /// Returns the number of the tag.
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

macro_rules! impl_from {
    ($tag:ty, $n:expr) => {
        impl From<$tag> for Tag {
            fn from(_: $tag) -> Self {
                Tag($n)
            }
        }
    };
}

/// Tag of the number `1`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag1;
impl_from!(Tag1, 1);

/// Tag of the number `2`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag2;
impl_from!(Tag2, 2);

/// Tag of the number `3`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag3;
impl_from!(Tag3, 3);

/// Tag of the number `4`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag4;
impl_from!(Tag4, 4);

/// Tag of the number `5`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag5;
impl_from!(Tag5, 5);

/// Tag of the number `6`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag6;
impl_from!(Tag6, 6);

/// Tag of the number `7`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag7;
impl_from!(Tag7, 7);

/// Tag of the number `8`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag8;
impl_from!(Tag8, 8);

/// Tag of the number `9`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag9;
impl_from!(Tag9, 9);

/// Tag of the number `10`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag10;
impl_from!(Tag10, 10);

/// Tag of the number `11`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag11;
impl_from!(Tag11, 11);

/// Tag of the number `12`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag12;
impl_from!(Tag12, 12);

/// Tag of the number `13`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag13;
impl_from!(Tag13, 13);

/// Tag of the number `14`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag14;
impl_from!(Tag14, 14);

/// Tag of the number `15`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag15;
impl_from!(Tag15, 15);

/// Tag of the number `16`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tag16;
impl_from!(Tag16, 16);
