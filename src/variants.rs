#[derive(Debug, Clone, Copy)]
pub enum Variant1<A> {
    A(A),
}

#[derive(Debug, Clone, Copy)]
pub enum Variant2<A, B> {
    A(A),
    B(B),
}

#[derive(Debug, Clone, Copy)]
pub enum Variant3<A, B, C> {
    A(A),
    B(B),
    C(C),
}

#[derive(Debug, Clone, Copy)]
pub enum Variant4<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
}

#[derive(Debug, Clone, Copy)]
pub enum Variant5<A, B, C, D, E> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
}

#[derive(Debug, Clone, Copy)]
pub enum Variant6<A, B, C, D, E, F> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
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
}
