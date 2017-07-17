use std::io::Take;
use futures::{Future, Poll};
use futures::future;

use Error;

pub type Finished<S, T> = future::Finished<(S, T), Error<S>>;

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
pub enum Phase1<A> {
    A(A),
}
impl<S, A> Future for Phase1<A>
where
    A: Future<Error = Error<S>>,
{
    type Item = Phase1<A::Item>;
    type Error = Error<S>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Phase1::A(ref mut f) => track!(f.poll())?.map(Phase1::A),
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
pub enum Phase3<A, B, C> {
    A(A),
    B(B),
    C(C),
}
impl<S, A, B, C> Future for Phase3<A, B, C>
where
    A: Future<Error = Error<S>>,
    B: Future<Error = Error<S>>,
    C: Future<Error = Error<S>>,
{
    type Item = Phase3<A::Item, B::Item, C::Item>;
    type Error = Error<S>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Phase3::A(ref mut f) => track!(f.poll())?.map(Phase3::A),
            Phase3::B(ref mut f) => track!(f.poll())?.map(Phase3::B),
            Phase3::C(ref mut f) => track!(f.poll())?.map(Phase3::C),
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

#[derive(Debug)]
pub enum Phase5<A, B, C, D, E> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
}
impl<S, A, B, C, D, E> Future for Phase5<A, B, C, D, E>
where
    A: Future<Error = Error<S>>,
    B: Future<Error = Error<S>>,
    C: Future<Error = Error<S>>,
    D: Future<Error = Error<S>>,
    E: Future<Error = Error<S>>,
{
    type Item = Phase5<A::Item, B::Item, C::Item, D::Item, E::Item>;
    type Error = Error<S>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Phase5::A(ref mut f) => track!(f.poll())?.map(Phase5::A),
            Phase5::B(ref mut f) => track!(f.poll())?.map(Phase5::B),
            Phase5::C(ref mut f) => track!(f.poll())?.map(Phase5::C),
            Phase5::D(ref mut f) => track!(f.poll())?.map(Phase5::D),
            Phase5::E(ref mut f) => track!(f.poll())?.map(Phase5::E),
        })
    }
}

#[derive(Debug)]
pub enum Phase6<A, B, C, D, E, F> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
}
impl<S, A, B, C, D, E, F> Future for Phase6<A, B, C, D, E, F>
where
    A: Future<Error = Error<S>>,
    B: Future<Error = Error<S>>,
    C: Future<Error = Error<S>>,
    D: Future<Error = Error<S>>,
    E: Future<Error = Error<S>>,
    F: Future<Error = Error<S>>,
{
    type Item = Phase6<A::Item, B::Item, C::Item, D::Item, E::Item, F::Item>;
    type Error = Error<S>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Phase6::A(ref mut f) => track!(f.poll())?.map(Phase6::A),
            Phase6::B(ref mut f) => track!(f.poll())?.map(Phase6::B),
            Phase6::C(ref mut f) => track!(f.poll())?.map(Phase6::C),
            Phase6::D(ref mut f) => track!(f.poll())?.map(Phase6::D),
            Phase6::E(ref mut f) => track!(f.poll())?.map(Phase6::E),
            Phase6::F(ref mut f) => track!(f.poll())?.map(Phase6::F),
        })
    }
}

#[derive(Debug)]
pub enum Phase7<A, B, C, D, E, F, G> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    G(G),
}
impl<S, A, B, C, D, E, F, G> Future for Phase7<A, B, C, D, E, F, G>
where
    A: Future<Error = Error<S>>,
    B: Future<Error = Error<S>>,
    C: Future<Error = Error<S>>,
    D: Future<Error = Error<S>>,
    E: Future<Error = Error<S>>,
    F: Future<Error = Error<S>>,
    G: Future<Error = Error<S>>,
{
    type Item = Phase7<A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item>;
    type Error = Error<S>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Phase7::A(ref mut f) => track!(f.poll())?.map(Phase7::A),
            Phase7::B(ref mut f) => track!(f.poll())?.map(Phase7::B),
            Phase7::C(ref mut f) => track!(f.poll())?.map(Phase7::C),
            Phase7::D(ref mut f) => track!(f.poll())?.map(Phase7::D),
            Phase7::E(ref mut f) => track!(f.poll())?.map(Phase7::E),
            Phase7::F(ref mut f) => track!(f.poll())?.map(Phase7::F),
            Phase7::G(ref mut f) => track!(f.poll())?.map(Phase7::G),
        })
    }
}

#[derive(Debug)]
pub enum Phase8<A, B, C, D, E, F, G, H> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    G(G),
    H(H),
}
impl<S, A, B, C, D, E, F, G, H> Future for Phase8<A, B, C, D, E, F, G, H>
where
    A: Future<Error = Error<S>>,
    B: Future<Error = Error<S>>,
    C: Future<Error = Error<S>>,
    D: Future<Error = Error<S>>,
    E: Future<Error = Error<S>>,
    F: Future<Error = Error<S>>,
    G: Future<Error = Error<S>>,
    H: Future<Error = Error<S>>,
{
    type Item = Phase8<A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item, H::Item>;
    type Error = Error<S>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(match *self {
            Phase8::A(ref mut f) => track!(f.poll())?.map(Phase8::A),
            Phase8::B(ref mut f) => track!(f.poll())?.map(Phase8::B),
            Phase8::C(ref mut f) => track!(f.poll())?.map(Phase8::C),
            Phase8::D(ref mut f) => track!(f.poll())?.map(Phase8::D),
            Phase8::E(ref mut f) => track!(f.poll())?.map(Phase8::E),
            Phase8::F(ref mut f) => track!(f.poll())?.map(Phase8::F),
            Phase8::G(ref mut f) => track!(f.poll())?.map(Phase8::G),
            Phase8::H(ref mut f) => track!(f.poll())?.map(Phase8::H),
        })
    }
}
