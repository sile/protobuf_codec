use futures::{Future, Poll};

use Error;
use variants::Variant2;

#[derive(Debug)]
pub enum Branch2<F0, F1> {
    A(F0),
    B(F1),
}
impl<S, F0, F1, V0, V1> Future for Branch2<F0, F1>
where
    F0: Future<Item = (S, V0), Error = Error<S>>,
    F1: Future<Item = (S, V1), Error = Error<S>>,
{
    type Item = (S, Variant2<V0, V1>);
    type Error = Error<S>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        panic!()
    }
}
