use std::io::Take;
use futures::{Future, Poll};
use futures::future;

use Error;

pub type Finished<S, T> = future::Finished<(S, T), Error<S>>;

#[derive(Debug)]
pub struct UnwrapTake<F>(pub F);
impl<R, V, F> Future for UnwrapTake<F>
where
    F: Future<Item = (Take<R>, V), Error = Error<Take<R>>>,
{
    type Item = (Take<R>, V);
    type Error = Error<R>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll().map_err(|e| {
            Error {
                stream: e.stream.into_inner(),
                error: e.error,
            }
        })
    }
}

#[derive(Debug)]
pub enum Phase2<A, B> {
    A(A),
    B(B),
}
impl<S, A, B> Future for Phase2<A, B>
where
    A: Future<Error = Error<S>>,
    B: Future<Error = Error<S>>,
{
    type Item = Phase2<A::Item, B::Item>;
    type Error = Error<S>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Phase2::A(ref mut f) => track!(f.poll())?.map(Phase2::A),
            Phase2::B(ref mut f) => track!(f.poll())?.map(Phase2::B),
        })
    }
}

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
