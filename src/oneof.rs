#[derive(Debug)]
pub enum OneOf2<A, B> {
    A(A),
    B(B),
    None,
}
impl<A, B> Default for OneOf2<A, B> {
    fn default() -> Self {
        OneOf2::None
    }
}

#[derive(Debug)]
pub enum OneOf3<A, B, C> {
    A(A),
    B(B),
    C(C),
    None,
}
impl<A, B, C> Default for OneOf3<A, B, C> {
    fn default() -> Self {
        OneOf3::None
    }
}

#[derive(Debug)]
pub enum OneOf4<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
    None,
}
impl<A, B, C, D> Default for OneOf4<A, B, C, D> {
    fn default() -> Self {
        OneOf4::None
    }
}

#[derive(Debug)]
pub enum OneOf5<A, B, C, D, E> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    None,
}
impl<A, B, C, D, E> Default for OneOf5<A, B, C, D, E> {
    fn default() -> Self {
        OneOf5::None
    }
}

#[derive(Debug)]
pub enum OneOf6<A, B, C, D, E, F> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    None,
}
impl<A, B, C, D, E, F> Default for OneOf6<A, B, C, D, E, F> {
    fn default() -> Self {
        OneOf6::None
    }
}

#[derive(Debug)]
pub enum OneOf7<A, B, C, D, E, F, G> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    G(G),
    None,
}
impl<A, B, C, D, E, F, G> Default for OneOf7<A, B, C, D, E, F, G> {
    fn default() -> Self {
        OneOf7::None
    }
}

#[derive(Debug)]
pub enum OneOf8<A, B, C, D, E, F, G, H> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    G(G),
    H(H),
    None,
}
impl<A, B, C, D, E, F, G, H> Default for OneOf8<A, B, C, D, E, F, G, H> {
    fn default() -> Self {
        OneOf8::None
    }
}
