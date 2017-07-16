use std::io::Write;
use futures::{Future, Poll, Async};

use {Encode, Error};
use fields;
use traits::SingularField;
use variants;

macro_rules! define_and_impl_encode_oneof {
    ($oneof:ident, $encoder:ident, $variant:ident, $($param:ident),*) => {
        #[derive(Debug)]
        pub enum $encoder<W, $($param),*>
        where
            W: Write,
            $($param: Encode<W>),*
        {
            None(Option<W>),
            $($param($param::Future)),*
        }
        impl<W, $($param),*> Future for $encoder<W, $($param),*>
        where
            W: Write,
            $($param: Encode<W> + SingularField),*
        {
            type Item = W;
            type Error = Error<W>;
            fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
                match *self {
                    $encoder::None(ref mut w) => {
                        let w = w.take().expect("Cannot poll EncodeOneof2 twice");
                        Ok(Async::Ready(w))
                    }
                    $($encoder::$param(ref mut f) => track!(f.poll())),*
                }
            }
        }
        impl<W, $($param),*> Encode<W> for fields::$oneof<$($param),*>
        where
            W: Write,
            $($param: Encode<W> + SingularField),*        
        {
            type Future = $encoder<W, $($param),*>;
            fn encode(self, writer: W) -> Self::Future {
                match self.field {
                    None => $encoder::None(Some(writer)),
                    $(Some(variants::$variant::$param(v)) => $encoder::$param(v.encode(writer))),*
                }
            }
            fn encoded_size(&self) -> u64 {
                match self.field {
                    None => 0,
                    $(Some(variants::$variant::$param(ref f)) => f.encoded_size()),*
                }
            }
        }
    }
}

define_and_impl_encode_oneof!(Oneof1, EncodeOneof1, Variant1, A);
define_and_impl_encode_oneof!(Oneof2, EncodeOneof2, Variant2, A, B);
define_and_impl_encode_oneof!(Oneof3, EncodeOneof3, Variant3, A, B, C);
define_and_impl_encode_oneof!(Oneof4, EncodeOneof4, Variant4, A, B, C, D);
define_and_impl_encode_oneof!(Oneof5, EncodeOneof5, Variant5, A, B, C, D, E);
define_and_impl_encode_oneof!(Oneof6, EncodeOneof6, Variant6, A, B, C, D, E, F);
define_and_impl_encode_oneof!(Oneof7, EncodeOneof7, Variant7, A, B, C, D, E, F, G);
define_and_impl_encode_oneof!(Oneof8, EncodeOneof8, Variant8, A, B, C, D, E, F, G, H);
