#![allow(clippy::single_match, clippy::block_in_if_condition_stmt)]
use crate::field::{FieldDecode, FieldEncode, RequiredFieldDecode, RequiredFieldEncode};
use crate::wire::Tag;
use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, Result, SizedEncode};

/// Value of `Oneof` that has 2-fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Branch2<A, B> {
    A(A),
    B(B),
}

/// Value of `Oneof` that has 3-fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Branch3<A, B, C> {
    A(A),
    B(B),
    C(C),
}

/// Value of `Oneof` that has 4-fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Branch4<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
}

/// Value of `Oneof` that has 5-fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Branch5<A, B, C, D, E> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
}

/// Value of `Oneof` that has 6-fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Branch6<A, B, C, D, E, F> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
}

/// Value of `Oneof` that has 7-fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Branch7<A, B, C, D, E, F, G> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    G(G),
}

/// Value of `Oneof` that has 8-fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    ($oneof:ident, [$($f:ident),*], [$($i:tt),*]) => {
        impl<$($f),*> Decode for Oneof<($($f),*,)>
        where
            $($f: RequiredFieldDecode),*
        {
            type Item = $oneof<$($f::Item),*>;

            fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
                if self.index == 0 {
                    return Ok(0)
                }
                match self.index - 1 {
                    $($i => track!(self.fields.$i.decode(buf, eos))),*,
                    _ => unreachable!()
                }
            }

            fn finish_decoding(&mut self) -> Result<Self::Item> {
                let i = self.index;
                self.index = 0;
                track_assert_ne!(i, 0, ErrorKind::InvalidInput, "No `Oneof` field");
                match i - 1 {
                    $($i => track!(self.fields.$i.finish_decoding()).map($oneof::$f)),*,
                    _ => unreachable!()
                }
            }

            fn is_idle(&self) -> bool {
                if self.index == 0 {
                    return false;
                }
                match self.index - 1 {
                    $($i => self.fields.$i.is_idle()),*,
                    _ => unreachable!(),
                }
            }

            fn requiring_bytes(&self) -> ByteCount {
                if self.index == 0 {
                    return ByteCount::Unknown;
                }
                match self.index - 1 {
                    $($i => self.fields.$i.requiring_bytes()),*,
                    _ => unreachable!()
                }
            }
        }
        impl<$($f),*> FieldDecode for Oneof<($($f),*,)>
        where
            $($f: RequiredFieldDecode),*
        {
            fn start_decoding(&mut self, tag: Tag) -> Result<bool> {
                if self.index != 0 {
                    match self.index - 1 {
                        $($i => track!(self.fields.$i.finish_decoding()).map(|_| ())?),*,
                        _ => {},
                    }
                    self.index = 0;
                }

                $(if track!(self.fields.$i.start_decoding(tag); tag)? {
                    self.index = $i + 1;
                    return Ok(true);
                })*
                Ok(false)
            }
        }
        impl<$($f),*> RequiredFieldDecode for Oneof<($($f),*,)>
        where
            $($f: RequiredFieldDecode),*
        {
            fn is_present(&self) -> bool {
                self.index != 0
            }
        }
    }
}

macro_rules! impl_field_encode {
    ($oneof:ident,[$($f:ident),*],[$($i:tt),*]) => {
        impl<$($f),*> Encode for Oneof<($($f),*,)>
        where
            $($f: RequiredFieldEncode),*
        {
            type Item = $oneof<$($f::Item),*>;

            fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
                if self.index == 0 {
                    return Ok(0);
                }
                match self.index - 1 {
                    $($i => track!(self.fields.$i.encode(buf, eos))),*,
                    _ => unreachable!()
                }
            }

            fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
                track_assert!(self.is_idle(), ErrorKind::EncoderFull);
                match item {
                    $($oneof::$f(v) => {
                        self.index = $i + 1;
                        track!(self.fields.$i.start_encoding(v))
                    }),*
                }
            }

            fn is_idle(&self) -> bool {
                if self.index == 0 {
                    return true;
                }
                match self.index - 1 {
                    $($i => self.fields.$i.is_idle()),*,
                    _ => unreachable!()
                }
            }

            fn requiring_bytes(&self) -> ByteCount {
                if self.index == 0 {
                    return ByteCount::Finite(0);
                }
                match self.index - 1 {
                    $($i => self.fields.$i.requiring_bytes()),*,
                    _ => unreachable!()
                }
            }
        }
        impl<$($f),*> FieldEncode for Oneof<($($f),*,)>
        where
            $($f: RequiredFieldEncode),*
        {
        }
        impl<$($f),*> RequiredFieldEncode for Oneof<($($f),*,)>
        where
            $($f: RequiredFieldEncode),*
        {
        }
        impl<$($f),*> SizedEncode for Oneof<($($f),*,)>
        where
            $($f: RequiredFieldEncode + SizedEncode),*
        {
            fn exact_requiring_bytes(&self) -> u64 {
                if self.index == 0 {
                    return 0;
                }
                match self.index - 1 {
                    $($i => self.fields.$i.exact_requiring_bytes()),*,
                    _ => unreachable!(),
                }
            }
        }
    };
}

impl_field_decode!(Branch2, [A, B], [0, 1]);
impl_field_decode!(Branch3, [A, B, C], [0, 1, 2]);
impl_field_decode!(Branch4, [A, B, C, D], [0, 1, 2, 3]);
impl_field_decode!(Branch5, [A, B, C, D, E], [0, 1, 2, 3, 4]);
impl_field_decode!(Branch6, [A, B, C, D, E, F], [0, 1, 2, 3, 4, 5]);
impl_field_decode!(Branch7, [A, B, C, D, E, F, G], [0, 1, 2, 3, 4, 5, 6]);
impl_field_decode!(Branch8, [A, B, C, D, E, F, G, H], [0, 1, 2, 3, 4, 5, 6, 7]);

impl_field_encode!(Branch2, [A, B], [0, 1]);
impl_field_encode!(Branch3, [A, B, C], [0, 1, 2]);
impl_field_encode!(Branch4, [A, B, C, D], [0, 1, 2, 3]);
impl_field_encode!(Branch5, [A, B, C, D, E], [0, 1, 2, 3, 4]);
impl_field_encode!(Branch6, [A, B, C, D, E, F], [0, 1, 2, 3, 4, 5]);
impl_field_encode!(Branch7, [A, B, C, D, E, F, G], [0, 1, 2, 3, 4, 5, 6]);
impl_field_encode!(Branch8, [A, B, C, D, E, F, G, H], [0, 1, 2, 3, 4, 5, 6, 7]);
