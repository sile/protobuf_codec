#[derive(Debug, Clone, Copy)]
pub enum Variant2<A, B> {
    A(A),
    B(B),
    None,
}
impl<A, B> Default for Variant2<A, B> {
    fn default() -> Self {
        Variant2::None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Variant3<A, B, C> {
    A(A),
    B(B),
    C(C),
    None,
}
impl<A, B, C> Default for Variant3<A, B, C> {
    fn default() -> Self {
        Variant3::None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Variant4<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
    None,
}
impl<A, B, C, D> Default for Variant4<A, B, C, D> {
    fn default() -> Self {
        Variant4::None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Variant5<A, B, C, D, E> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    None,
}
impl<A, B, C, D, E> Default for Variant5<A, B, C, D, E> {
    fn default() -> Self {
        Variant5::None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Variant6<A, B, C, D, E, F> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    None,
}
impl<A, B, C, D, E, F> Default for Variant6<A, B, C, D, E, F> {
    fn default() -> Self {
        Variant6::None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Variant7<A, B, C, D, E, F, G> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
    G(G),
    None,
}
impl<A, B, C, D, E, F, G> Default for Variant7<A, B, C, D, E, F, G> {
    fn default() -> Self {
        Variant7::None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Variant8<A, B, C, D, E, F, G, H> {
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
impl<A, B, C, D, E, F, G, H> Default for Variant8<A, B, C, D, E, F, G, H> {
    fn default() -> Self {
        Variant8::None
    }
}
