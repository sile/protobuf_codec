use bytecodec::{ByteCount, Encode, Eos, ErrorKind, ExactBytesEncode, Result};

use field::{FieldDecode, FieldEncode};
use tag::Tag;
use wire::WireType;

/// Decoder and encoder for multiple fields.
#[derive(Debug, Default)]
pub struct Fields<F> {
    fields: F,
}
impl<F> Fields<F> {
    /// Makes a new `Fields` instance.
    pub fn new(fields: F) -> Self {
        Fields { fields }
    }
}

macro_rules! impl_field_decode {
    ([$($f:ident),*],[$($i:tt),*]) => {
        impl<$($f),*> FieldDecode for Fields<($($f),*,)>
        where
            $($f: FieldDecode),*,
        {
            type Item = ($($f::Item),*,);

            fn start_decoding(&mut self, tag: Tag, wire_type: WireType) -> Result<bool> {
                $(if track!(self.fields.$i.start_decoding(tag, wire_type), "i={}", $i)? {
                    return Ok(true);
                })*
                Ok(false)
            }

            fn field_decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
                $(if self.fields.$i.is_decoding() {
                    return track!(self.fields.0.field_decode(buf, eos), "i={}", $i);
                })*
                track_panic!(ErrorKind::Other)
            }

            fn is_decoding(&self) -> bool {
                $(self.fields.$i.is_decoding())||*
            }

            fn finish_decoding(&mut self) -> Result<Self::Item> {
                Ok(($(track!(self.fields.$i.finish_decoding(), "i={}", $i)?),*,))
            }

            fn requiring_bytes(&self) -> ByteCount {
                $(if self.fields.$i.is_decoding() {
                    return self.fields.$i.requiring_bytes();
                })*
                ByteCount::Unknown
            }

            fn merge_fields(old: &mut Self::Item, new: Self::Item) {
                $($f::merge_fields(&mut old.$i, new.$i));*
            }
        }
    };
}

impl_field_decode!([A], [0]);
impl_field_decode!([A, B], [0, 1]);
impl_field_decode!([A, B, C], [0, 1, 2]);
impl_field_decode!([A, B, C, D], [0, 1, 2, 3]);
impl_field_decode!([A, B, C, D, E], [0, 1, 2, 3, 4]);
impl_field_decode!([A, B, C, D, E, F], [0, 1, 2, 3, 4, 5]);
impl_field_decode!([A, B, C, D, E, F, G], [0, 1, 2, 3, 4, 5, 6]);
impl_field_decode!([A, B, C, D, E, F, G, H], [0, 1, 2, 3, 4, 5, 6, 7]);

macro_rules! impl_field_encode {
    ([$($f:ident),*], [$($i:tt),*], $last:tt) => {
        impl<$($f),*> Encode for Fields<($($f),*,)>
        where
            $($f: FieldEncode),*
        {
            type Item = ($($f::Item),*,);

            fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
                let mut offset = 0;
                $(if !self.fields.$i.is_idle() {
                    offset += track!(self.fields.$i.encode(&mut buf[offset..], eos), "i={}", $i)?;
                    if !self.fields.$i.is_idle() {
                        return Ok(offset);
                    }
                });*
                Ok(offset)
            }

            fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
                $(track!(self.fields.$i.start_encoding(item.$i), "i={}", $i)?;)*
                Ok(())
            }

            fn is_idle(&self) -> bool {
                self.fields.$last.is_idle()
            }

            fn requiring_bytes(&self) -> ByteCount {
                ByteCount::Finite(0)$(.add_for_encoding(self.fields.$i.requiring_bytes()))*
            }
        }
        impl<$($f),*> ExactBytesEncode for Fields<($($f),*,)>
        where
            $($f: FieldEncode + ExactBytesEncode),*
        {
            fn exact_requiring_bytes(&self) -> u64 {
                0 $(+ self.fields.$i.exact_requiring_bytes())*
            }
        }
        impl<$($f),*> FieldEncode for Fields<($($f),*,)>
        where
            $($f: FieldEncode),*
        {
        }
    };
}

impl_field_encode!([A], [0], 0);
impl_field_encode!([A, B], [0, 1], 1);
impl_field_encode!([A, B, C], [0, 1, 2], 2);
impl_field_encode!([A, B, C, D], [0, 1, 2, 3], 3);
impl_field_encode!([A, B, C, D, E], [0, 1, 2, 3, 4], 4);
impl_field_encode!([A, B, C, D, E, F], [0, 1, 2, 3, 4, 5], 5);
impl_field_encode!([A, B, C, D, E, F, G], [0, 1, 2, 3, 4, 5, 6], 6);
impl_field_encode!([A, B, C, D, E, F, G, H], [0, 1, 2, 3, 4, 5, 6, 7], 7);
