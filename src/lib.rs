extern crate byteorder;
extern crate futures;
#[macro_use]
extern crate trackable;

pub use traits::{Decode, Encode, Message};

pub use error::{Error, ErrorKind};

macro_rules! failed {
    ($stream:expr, $kind:expr) => {
        {
            use trackable::error::ErrorKindExt;
            return Err(track!(::Error{stream: $stream, error: $kind.error().into()}));
        }
    };
    ($stream:expr, $kind:expr, $($arg:tt),*) => {
        {
            use trackable::error::ErrorKindExt;
            let error = $kind.cause(format!($($arg),*)).into();
            return Err(track!(::Error{stream: $stream, error}));
        }
    }
}
macro_rules! failed_by_error {
    ($stream:expr, $kind:expr, $cause:expr) => {
        {
            use trackable::error::ErrorKindExt;
            return Err(track!(::Error{stream: $stream, error: $kind.cause($cause).into()}));
        }
    }
}

pub mod fields;
pub mod future;
pub mod traits;
pub mod tags;
pub mod types;
pub mod variants;
pub mod wire;

mod error;
mod message;

pub type Result<T> = std::result::Result<T, trackable::error::TrackableError<ErrorKind>>;

#[cfg(test)]
mod test {
    use futures::Future;

    use {Message, Result};
    use fields::Field;
    use tags::{Tag1, Tag2};
    use types::Int32;

    #[test]
    fn encode_works() {
        let value = (
            Field {
                tag: Tag1,
                value: Int32(150),
            },
            Field {
                tag: Tag2,
                value: Int32(150),
            },
        );
        let bytes = track_try_unwrap!(value.encode_message(Vec::new()).wait());
        assert_eq!(bytes, [0x08, 0x96, 0x01, 0x10, 0x96, 0x01]);
    }

    #[test]
    fn decode_works() {
        type M = (Field<Tag1, Int32>, Field<Tag2, Int32>);
        let bytes = [0x08, 0x96, 0x01, 0x10, 0x96, 0x01];
        let (_, m) = track_try_unwrap!(M::decode_message(&bytes[..]).wait());
        assert_eq!(m.0.value.0, 150);
        assert_eq!(m.1.value.0, 150);
    }

    #[test]
    fn derived_message_works() {
        #[derive(Debug, Default, PartialEq, Eq)]
        struct Foo {
            bar: i32,
            baz: i32,
        }
        impl Message for Foo {
            type Base = (Field<Tag1, Int32>, Field<Tag2, Int32>);
            fn from_base(base: Self::Base) -> Result<Self> {
                Ok(Foo {
                    bar: base.0.value.0,
                    baz: base.1.value.0,
                })
            }
            fn into_base(self) -> Self::Base {
                (Int32(self.bar).into(), Int32(self.baz).into())
            }
        }

        let m = Foo { bar: 150, baz: 150 };
        let bytes = track_try_unwrap!(m.encode_message(Vec::new()).wait());
        assert_eq!(bytes, [0x08, 0x96, 0x01, 0x10, 0x96, 0x01]);

        let (_, m) = track_try_unwrap!(Foo::decode_message(&bytes[..]).wait());
        assert_eq!(m, Foo { bar: 150, baz: 150 });
    }
}
