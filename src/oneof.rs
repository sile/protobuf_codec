use bytecodec::{ByteCount, Encode, Eos, ErrorKind, ExactBytesEncode, Result};

use field::{FieldDecode, FieldEncode, OneOfFieldDecode, OneOfFieldEncode};
use tag::Tag;
use wire::WireType;

#[derive(Debug)]
pub enum OneOf1<A> {
    A(A),
    None,
}
impl<A> Default for OneOf1<A> {
    fn default() -> Self {
        OneOf1::None
    }
}

#[derive(Debug)]
pub enum OneOf2<A, B> {
    A(A),
    B(B),
    None,
}
impl<A, B> Default for OneOf2<A, B> {
    fn default() -> Self {
        OneOf2::None
    }
}

#[derive(Debug)]
pub enum OneOf3<A, B, C> {
    A(A),
    B(B),
    C(C),
    None,
}
impl<A, B, C> Default for OneOf3<A, B, C> {
    fn default() -> Self {
        OneOf3::None
    }
}

#[derive(Debug)]
pub enum OneOf4<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
    None,
}
impl<A, B, C, D> Default for OneOf4<A, B, C, D> {
    fn default() -> Self {
        OneOf4::None
    }
}

#[derive(Debug)]
pub enum OneOf5<A, B, C, D, E> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    None,
}
impl<A, B, C, D, E> Default for OneOf5<A, B, C, D, E> {
    fn default() -> Self {
        OneOf5::None
    }
}

#[derive(Debug)]
pub enum OneOf6<A, B, C, D, E, F> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    None,
}
impl<A, B, C, D, E, F> Default for OneOf6<A, B, C, D, E, F> {
    fn default() -> Self {
        OneOf6::None
    }
}

#[derive(Debug)]
pub enum OneOf7<A, B, C, D, E, F, G> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    G(G),
    None,
}
impl<A, B, C, D, E, F, G> Default for OneOf7<A, B, C, D, E, F, G> {
    fn default() -> Self {
        OneOf7::None
    }
}

#[derive(Debug)]
pub enum OneOf8<A, B, C, D, E, F, G, H> {
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
impl<A, B, C, D, E, F, G, H> Default for OneOf8<A, B, C, D, E, F, G, H> {
    fn default() -> Self {
        OneOf8::None
    }
}

macro_rules! impl_field_decode {
    ($oneof:ident, [$($f:ident),*], [$($i:expr),*], [$($j:tt),*]) => {
        impl<$($f),*> FieldDecode for OneOf<($($f),*,)>
        where
            $($f: OneOfFieldDecode),*
        {
            type Item = $oneof<$($f::Item),*>;

            fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
                match self.index {
                    $($i => track!(self.fields.$j.finish_decoding()).map(|_| ())?),*,
                    _ => {},
                }

                $(if track!(self.fields.$j.start_decoding(tag, wire_type))? {
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
        impl<$($f),*> OneOfFieldDecode for OneOf<($($f),*,)>
        where
            $($f: OneOfFieldDecode),*
        {
        }
    }
}

#[derive(Debug, Default)]
pub struct OneOf<F> {
    fields: F,
    index: usize,
}
impl_field_decode!(OneOf1, [A], [1], [0]);
impl_field_decode!(OneOf2, [A, B], [1, 2], [0, 1]);
impl_field_decode!(OneOf3, [A, B, C], [1, 2, 3], [0, 1, 2]);
impl_field_decode!(OneOf4, [A, B, C, D], [1, 2, 3, 4], [0, 1, 2, 3]);
impl_field_decode!(OneOf5, [A, B, C, D, E], [1, 2, 3, 4, 5], [0, 1, 2, 3, 4]);
impl_field_decode!(
    OneOf6,
    [A, B, C, D, E, F],
    [1, 2, 3, 4, 5, 6],
    [0, 1, 2, 3, 4, 5]
);
impl_field_decode!(
    OneOf7,
    [A, B, C, D, E, F, G],
    [1, 2, 3, 4, 5, 6, 7],
    [0, 1, 2, 3, 4, 5, 6]
);
impl_field_decode!(
    OneOf8,
    [A, B, C, D, E, F, G, H],
    [1, 2, 3, 4, 5, 6, 7, 8],
    [0, 1, 2, 3, 4, 5, 6, 7]
);
impl<F0, F1> Encode for OneOf<(F0, F1)>
where
    F0: OneOfFieldEncode,
    F1: OneOfFieldEncode,
{
    type Item = OneOf2<F0::Item, F1::Item>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        match self.index {
            0 => Ok(0),
            1 => track!(self.fields.0.encode(buf, eos)),
            2 => track!(self.fields.1.encode(buf, eos)),
            _ => unreachable!(),
        }
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        match item {
            OneOf2::A(v) => {
                self.index = 1;
                track!(self.fields.0.start_encoding(v))
            }
            OneOf2::B(v) => {
                self.index = 2;
                track!(self.fields.1.start_encoding(v))
            }
            OneOf2::None => Ok(()),
        }
    }

    fn is_idle(&self) -> bool {
        match self.index {
            0 => true,
            1 => self.fields.0.is_idle(),
            2 => self.fields.1.is_idle(),
            _ => unreachable!(),
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        match self.index {
            0 => ByteCount::Finite(0),
            1 => self.fields.0.requiring_bytes(),
            2 => self.fields.1.requiring_bytes(),
            _ => unreachable!(),
        }
    }
}
impl<F0, F1> FieldEncode for OneOf<(F0, F1)>
where
    F0: OneOfFieldEncode,
    F1: OneOfFieldEncode,
{
}
impl<F0, F1> OneOfFieldEncode for OneOf<(F0, F1)>
where
    F0: OneOfFieldEncode,
    F1: OneOfFieldEncode,
{
}
impl<F0, F1> ExactBytesEncode for OneOf<(F0, F1)>
where
    F0: OneOfFieldEncode + ExactBytesEncode,
    F1: OneOfFieldEncode + ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        match self.index {
            0 => 0,
            1 => self.fields.0.exact_requiring_bytes(),
            2 => self.fields.1.exact_requiring_bytes(),
            _ => unreachable!(),
        }
    }
}
