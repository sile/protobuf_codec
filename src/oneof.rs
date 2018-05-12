#![cfg_attr(feature = "cargo-clippy", allow(single_match, block_in_if_condition_stmt))]
use bytecodec::{ByteCount, Encode, Eos, ErrorKind, ExactBytesEncode, Result};

use field::{FieldDecode, FieldEncode, OneofFieldDecode, OneofFieldEncode};
use wire::Tag;

/// Value of `Oneof` that has 1-field.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Branch1<A> {
    A(A),
    None,
}
impl<A> Default for Branch1<A> {
    fn default() -> Self {
        Branch1::None
    }
}

/// Value of `Oneof` that has 2-fields.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Branch2<A, B> {
    A(A),
    B(B),
    None,
}
impl<A, B> Default for Branch2<A, B> {
    fn default() -> Self {
        Branch2::None
    }
}

/// Value of `Oneof` that has 3-fields.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Branch3<A, B, C> {
    A(A),
    B(B),
    C(C),
    None,
}
impl<A, B, C> Default for Branch3<A, B, C> {
    fn default() -> Self {
        Branch3::None
    }
}

/// Value of `Oneof` that has 4-fields.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Branch4<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
    None,
}
impl<A, B, C, D> Default for Branch4<A, B, C, D> {
    fn default() -> Self {
        Branch4::None
    }
}

/// Value of `Oneof` that has 5-fields.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Branch5<A, B, C, D, E> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    None,
}
impl<A, B, C, D, E> Default for Branch5<A, B, C, D, E> {
    fn default() -> Self {
        Branch5::None
    }
}

/// Value of `Oneof` that has 6-fields.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Branch6<A, B, C, D, E, F> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    None,
}
impl<A, B, C, D, E, F> Default for Branch6<A, B, C, D, E, F> {
    fn default() -> Self {
        Branch6::None
    }
}

/// Value of `Oneof` that has 7-fields.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Branch7<A, B, C, D, E, F, G> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    G(G),
    None,
}
impl<A, B, C, D, E, F, G> Default for Branch7<A, B, C, D, E, F, G> {
    fn default() -> Self {
        Branch7::None
    }
}

/// Value of `Oneof` that has 8-fields.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Branch8<A, B, C, D, E, F, G, H> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    G(G),
    H(H),
    None,
}
impl<A, B, C, D, E, F, G, H> Default for Branch8<A, B, C, D, E, F, G, H> {
    fn default() -> Self {
        Branch8::None
    }
}

/// Decoder and encoder for `Oneof` fields.
#[derive(Debug, Default)]
pub struct Oneof<F> {
    fields: F,
    index: usize,
}
impl<F> Oneof<F> {
    /// Makes a new `Oneof` instance.
    pub fn new(fields: F) -> Self {
        Oneof { fields, index: 0 }
    }
}

macro_rules! impl_field_decode {
    ($oneof:ident, [$($f:ident),*], [$($i:expr),*], [$($j:tt),*]) => {
        impl<$($f),*> FieldDecode for Oneof<($($f),*,)>
        where
            $($f: OneofFieldDecode),*
        {
            type Item = $oneof<$($f::Item),*>;
 
            fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
                match self.index {
                    $($i => track!(self.fields.$j.finish_decoding()).map(|_| ())?),*,
                    _ => {},
                }

                $(if track!(self.fields.$j.start_decoding(tag); tag)? {
                    self.index = $i;
                    return Ok(true);
                })*
                Ok(false)
            }

            fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
                match self.index {
                    $($i => track!(self.fields.$j.field_decode(buf, eos))),*,
                    _ => Ok(0),
                }
            }

            fn is_decoding(&self) -> bool {
                match self.index {
                    $($i => self.fields.$j.is_decoding()),*,
                    _ => false,
                }
            }

            fn finish_decoding(&mut self) -> Result<Self::Item> {
                let i = self.index;
                self.index = 0;
                match i {
                    $($i => track!(self.fields.$j.finish_decoding()).map($oneof::$f)),*,
                    _ => Ok($oneof::None),
                }
            }

            fn requiring_bytes(&self) -> ByteCount {
                match self.index {
                    $($i => self.fields.$j.requiring_bytes()),*,
                    _ => ByteCount::Unknown,
                }
            }

            fn merge_fields(old: &mut Self::Item, new: Self::Item) {
                *old = new;
            }
        }
        impl<$($f),*> OneofFieldDecode for Oneof<($($f),*,)>
        where
            $($f: OneofFieldDecode),*
        {
        }
    }
}

macro_rules! impl_field_encode {
    ($oneof:ident,[$($f:ident),*],[$($i:expr),*],[$($j:tt),*]) => {
        impl<$($f),*> Encode for Oneof<($($f),*,)>
        where
            $($f: OneofFieldEncode),*
        {
            type Item = $oneof<$($f::Item),*>;

            fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
                match self.index {
                    $($i => track!(self.fields.$j.encode(buf, eos))),*,
                    _ => Ok(0),
                }
            }

            fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
                track_assert!(self.is_idle(), ErrorKind::EncoderFull);
                match item {
                    $($oneof::$f(v) => {
                        self.index = $i;
                        track!(self.fields.$j.start_encoding(v))
                    }),*
                    $oneof::None => Ok(()),
                }
            }

            fn is_idle(&self) -> bool {
                match self.index {
                    $($i => self.fields.$j.is_idle()),*,
                    _ => true,
                }
            }

            fn requiring_bytes(&self) -> ByteCount {
                match self.index {
                    $($i => self.fields.$j.requiring_bytes()),*,
                    _ => ByteCount::Finite(0),
                }
            }
        }
        impl<$($f),*> FieldEncode for Oneof<($($f),*,)>
        where
            $($f: OneofFieldEncode),*
        {
        }
        impl<$($f),*> OneofFieldEncode for Oneof<($($f),*,)>
        where
            $($f: OneofFieldEncode),*
        {
        }
        impl<$($f),*> ExactBytesEncode for Oneof<($($f),*,)>
        where
            $($f: OneofFieldEncode + ExactBytesEncode),*
        {
            fn exact_requiring_bytes(&self) -> u64 {
                match self.index {
                    $($i => self.fields.$j.exact_requiring_bytes()),*,
                    _ => 0,
                }
            }
        }
    };
}

impl_field_decode!(Branch1, [A], [1], [0]);
impl_field_decode!(Branch2, [A, B], [1, 2], [0, 1]);
impl_field_decode!(Branch3, [A, B, C], [1, 2, 3], [0, 1, 2]);
impl_field_decode!(Branch4, [A, B, C, D], [1, 2, 3, 4], [0, 1, 2, 3]);
impl_field_decode!(Branch5, [A, B, C, D, E], [1, 2, 3, 4, 5], [0, 1, 2, 3, 4]);
impl_field_decode!(
    Branch6,
    [A, B, C, D, E, F],
    [1, 2, 3, 4, 5, 6],
    [0, 1, 2, 3, 4, 5]
);
impl_field_decode!(
    Branch7,
    [A, B, C, D, E, F, G],
    [1, 2, 3, 4, 5, 6, 7],
    [0, 1, 2, 3, 4, 5, 6]
);
impl_field_decode!(
    Branch8,
    [A, B, C, D, E, F, G, H],
    [1, 2, 3, 4, 5, 6, 7, 8],
    [0, 1, 2, 3, 4, 5, 6, 7]
);

impl_field_encode!(Branch1, [A], [1], [0]);
impl_field_encode!(Branch2, [A, B], [1, 2], [0, 1]);
impl_field_encode!(Branch3, [A, B, C], [1, 2, 3], [0, 1, 2]);
impl_field_encode!(Branch4, [A, B, C, D], [1, 2, 3, 4], [0, 1, 2, 3]);
impl_field_encode!(Branch5, [A, B, C, D, E], [1, 2, 3, 4, 5], [0, 1, 2, 3, 4]);
impl_field_encode!(
    Branch6,
    [A, B, C, D, E, F],
    [1, 2, 3, 4, 5, 6],
    [0, 1, 2, 3, 4, 5]
);
impl_field_encode!(
    Branch7,
    [A, B, C, D, E, F, G],
    [1, 2, 3, 4, 5, 6, 7],
    [0, 1, 2, 3, 4, 5, 6]
);
impl_field_encode!(
    Branch8,
    [A, B, C, D, E, F, G, H],
    [1, 2, 3, 4, 5, 6, 7, 8],
    [0, 1, 2, 3, 4, 5, 6, 7]
);
