use std::io::Write;
use futures::Future;

use {Result, Error};
use traits::Pattern;

pub mod futures;

mod fields;
mod types;
mod wires;

pub trait Encode<W: Write>: Pattern {
    type Future: Future<Item = W, Error = Error<W>>;
    fn encode(writer: W, value: Self::Value) -> Self::Future;
    fn encoded_size(value: &Self::Value) -> u64;
    fn sync_encode(writer: W, value: Self::Value) -> Result<()> {
        Self::encode(writer, value).wait().map(|_| ()).map_err(
            |e| e.error,
        )
    }
}
impl<W: Write> Encode<W> for Vec<u8> {
    type Future = futures::WriteBytes<W, Vec<u8>>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        futures::WriteBytes::new(writer, value)
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        value.len() as u64
    }
}

#[cfg(test)]
mod test {
    use {Message, Encode};
    use fields::Field;
    use tags::{Tag1, Tag2};
    use types::Int32;

    #[test]
    fn it_works() {
        type M = Message<(Field<Tag1, Int32>, Field<Tag2, Int32>)>;
        let value = (150, 150);
        let mut bytes = Vec::new();
        track_try_unwrap!(M::sync_encode(&mut bytes, value));
        assert_eq!(bytes, [0x08, 0x96, 0x01, 0x10, 0x96, 0x01]);
    }
}
