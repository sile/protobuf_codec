use std::io::Take;
use futures::{Future, Poll};
use futures::future;

use Error;

pub type Finished<S, T> = future::Finished<(S, T), Error<S>>;

#[derive(Debug)]
pub enum Phase4<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
}
impl<S, A, B, C, D> Future for Phase4<A, B, C, D>
where
    A: Future<Error = Error<S>>,
    B: Future<Error = Error<S>>,
    C: Future<Error = Error<S>>,
    D: Future<Error = Error<S>>,
{
    type Item = Phase4<A::Item, B::Item, C::Item, D::Item>;
    type Error = Error<S>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Phase4::A(ref mut f) => track!(f.poll())?.map(Phase4::A),
            Phase4::B(ref mut f) => track!(f.poll())?.map(Phase4::B),
            Phase4::C(ref mut f) => track!(f.poll())?.map(Phase4::C),
            Phase4::D(ref mut f) => track!(f.poll())?.map(Phase4::D),
        })
    }
}
