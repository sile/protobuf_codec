use std::io::Take;
use futures::{Future, Poll};

use Error;

#[derive(Debug)]
pub struct WithState<F, T> {
    pub future: F,
    pub state: Option<T>,
}
impl<F, T> WithState<F, T> {
    pub fn new(future: F, state: T) -> Self {
        WithState {
            future,
            state: Some(state),
        }
    }
}
impl<F: Future, T> Future for WithState<F, T> {
    type Item = (F::Item, T);
    type Error = F::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(self.future.poll()?.map(|v| {
            let s = self.state.take().expect("Cannot poll WithState twice");
            (v, s)
        }))
    }
}

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
