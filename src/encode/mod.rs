use std::io::Write;
use futures::Future;
use trackable::error::TrackableError;

use {Result, Error, ErrorKind};

pub mod futures;

mod wires;

pub trait Encode<W: Write> {
    type Value;
    type Future: Future<Item = W, Error = Error<W>>;
    fn encode(writer: W, value: Self::Value) -> Self::Future;
    fn encoded_size(value: &Self::Value) -> u64;
    fn sync_encode(writer: W, value: Self::Value) -> Result<()> {
        Self::encode(writer, value).wait().map(|_| ()).map_err(
            |e| e.error,
        )
    }
}
