use std::marker::PhantomData;
use futures::Future;

#[derive(Debug)]
pub struct DecodeFrom<R, T, F> {
    future: F,
    _phantom: PhantomData<(R, T)>,
}
impl<R, T, F> DecodeFrom<R, T, F>
where
    T: From<F>,
    F: Decode<R>,
{
}
