use std::io::Read;
use futures::{Future, Poll};

use Error;
use fields;
use traits::{DecodeField, SingularField};
use variants;
use wire::WireType;

macro_rules! define_and_impl_decode_oneof {
    ($oneof:ident, $decoder:ident, $variant:ident, $($param:ident),*) => {
        #[derive(Debug)]
        pub enum $decoder<R, $($param),*>
        where
            R: Read,
            $($param: DecodeField<R>),*
        {
            $($param($param::Future)),*
        }

        impl<R, $($param),*> Future for $decoder<R, $($param),*>
        where
            R: Read,
            $($param: DecodeField<R> + SingularField),*
        {
            type Item = (R, fields::$oneof<$($param),*>);
            type Error = Error<R>;
            fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
                Ok(match *self {
                    $($decoder::$param(ref mut f) => {
                        track!(f.poll())?.map(|(r, v)| {
                            (r, From::from(Some(variants::$variant::$param(v))))
                        })
                    }),*
                })
            }
        }

        impl<R, $($param),*> DecodeField<R> for fields::$oneof<$($param),*>
        where
            R: Read,
            $($param: DecodeField<R> + SingularField),*
        {
            type Future = $decoder<R, $($param),*>;
            fn is_target(tag: u32) -> bool {
                $($param::is_target(tag))||*
            }
            fn decode_field(
                self,
                reader: R,
                tag: u32,
                wire_type: WireType,
            ) -> Result<Self::Future, Error<R>> {
                if false {
                    unreachable!()
                }
                $(else if $param::is_target(tag) {
                    let f = track!($param::default().decode_field(reader, tag, wire_type))?;
                    Ok($decoder::$param(f))
                })*
                else {
                    unreachable!()
                }
            }
        }
    }
}

define_and_impl_decode_oneof!(Oneof1, DecodeOneof1, Variant1, A);
define_and_impl_decode_oneof!(Oneof2, DecodeOneof2, Variant2, A, B);
define_and_impl_decode_oneof!(Oneof3, DecodeOneof3, Variant3, A, B, C);
define_and_impl_decode_oneof!(Oneof4, DecodeOneof4, Variant4, A, B, C, D);
define_and_impl_decode_oneof!(Oneof5, DecodeOneof5, Variant5, A, B, C, D, E);
define_and_impl_decode_oneof!(Oneof6, DecodeOneof6, Variant6, A, B, C, D, E, F);
define_and_impl_decode_oneof!(Oneof7, DecodeOneof7, Variant7, A, B, C, D, E, F, G);
define_and_impl_decode_oneof!(Oneof8, DecodeOneof8, Variant8, A, B, C, D, E, F, G, H);
