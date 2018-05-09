use bytecodec::{ByteCount, Encode, Eos, ErrorKind, ExactBytesEncode, Result};

use field::{FieldDecode, FieldEncode, SingularFieldDecode, SingularFieldEncode};
use tag::Tag;
use wire::WireType;

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

#[derive(Debug, Default)]
pub struct OneOfDecoder<D> {
    decoders: D,
    index: usize,
}
impl<D0, D1> FieldDecode for OneOfDecoder<(D0, D1)>
where
    D0: SingularFieldDecode,
    D1: SingularFieldDecode,
{
    type Item = OneOf2<D0::Item, D1::Item>;

    fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
        if track!(self.decoders.0.start_decoding(tag, wire_type))? {
            self.index = 1;
            Ok(true)
        } else if track!(self.decoders.1.start_decoding(tag, wire_type))? {
            self.index = 2;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        if self.decoders.0.is_decoding() {
            track!(self.decoders.0.field_decode(buf, eos))
        } else if self.decoders.1.is_decoding() {
            track!(self.decoders.1.field_decode(buf, eos))
        } else {
            Ok(0)
        }
    }

    fn is_decoding(&self) -> bool {
        self.decoders.0.is_decoding() || self.decoders.1.is_decoding()
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let v0 = track!(self.decoders.0.finish_decoding())?; // TODO: return Option<_>
        let v1 = track!(self.decoders.1.finish_decoding())?; // TODO: return Option<_>
        let i = self.index;
        self.index = 0;
        match i {
            0 => Ok(OneOf2::None),
            1 => Ok(OneOf2::A(v0)),
            2 => Ok(OneOf2::B(v1)),
            _ => unreachable!(),
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.decoders.0.is_decoding() {
            self.decoders.0.requiring_bytes()
        } else if self.decoders.1.is_decoding() {
            self.decoders.1.requiring_bytes()
        } else {
            ByteCount::Unknown
        }
    }

    fn merge(&self, _old: Self::Item, new: Self::Item) -> Self::Item {
        // TODO
        new
    }
}

#[derive(Debug, Default)]
pub struct OneOfEncoder<E> {
    encoders: E,
    index: usize,
}
impl<E0, E1> Encode for OneOfEncoder<(E0, E1)>
where
    E0: SingularFieldEncode,
    E1: SingularFieldEncode,
{
    type Item = OneOf2<E0::Item, E1::Item>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        match self.index {
            0 => Ok(0),
            1 => {
                let size = track!(self.encoders.0.encode(buf, eos))?;
                if self.encoders.0.is_idle() {
                    self.index = 0;
                }
                Ok(size)
            }
            2 => {
                let size = track!(self.encoders.1.encode(buf, eos))?;
                if self.encoders.1.is_idle() {
                    self.index = 0;
                }
                Ok(size)
            }
            _ => unreachable!(),
        }
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        match item {
            OneOf2::A(v) => {
                self.index = 1;
                track!(self.encoders.0.start_encoding(v))
            }
            OneOf2::B(v) => {
                self.index = 2;
                track!(self.encoders.1.start_encoding(v))
            }
            OneOf2::None => Ok(()),
        }
    }

    fn is_idle(&self) -> bool {
        self.index == 0
    }

    fn requiring_bytes(&self) -> ByteCount {
        match self.index {
            0 => ByteCount::Finite(0),
            1 => self.encoders.0.requiring_bytes(),
            2 => self.encoders.1.requiring_bytes(),
            _ => unreachable!(),
        }
    }
}
impl<E0, E1> ExactBytesEncode for OneOfEncoder<(E0, E1)>
where
    E0: SingularFieldEncode + ExactBytesEncode,
    E1: SingularFieldEncode + ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        match self.index {
            0 => 0,
            1 => self.encoders.0.exact_requiring_bytes(),
            2 => self.encoders.1.exact_requiring_bytes(),
            _ => unreachable!(),
        }
    }
}
impl<E0, E1> FieldEncode for OneOfEncoder<(E0, E1)>
where
    E0: SingularFieldEncode,
    E1: SingularFieldEncode,
{
}
