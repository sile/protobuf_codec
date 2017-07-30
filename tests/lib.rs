extern crate futures;
extern crate pbcodec;
#[macro_use]
extern crate trackable;

use futures::Future;
use pbcodec::{Encode, Decode};
use pbcodec::fields::PackedRepeatedField;
use pbcodec::tags::Tag1;
use pbcodec::types::Int32;

#[test]
fn packed_repeated_works() {
    type M = (PackedRepeatedField<Tag1, Int32>,);

    let v: M = (vec![Int32(0), Int32(1), Int32(2)].into(),);
    let bytes = track_try_unwrap!(v.encode(Vec::new()).wait());
    assert_eq!(bytes, [10, 3, 0, 1, 2]);

    let (_, m) = track_try_unwrap!(M::decode(&bytes[..]).wait());
    assert_eq!(
        m.0.values.into_iter().map(|v| v.0).collect::<Vec<_>>(),
        [0, 1, 2]
    );
}
