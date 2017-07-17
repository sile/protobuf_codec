use {Result, Message};

macro_rules! impl_anonymous_message {
    ($($param:ident),*) => {
        impl<$($param),*> Message for ($($param),*,)
            where
            $($param: Default),*
        {
            type Base = Self;
            fn from_base(base: Self::Base) -> Result<Self> {
                Ok(base)
            }
            fn into_base(self) -> Self::Base {
                self
            }
        }
    }
}

impl_anonymous_message!(A);
impl_anonymous_message!(A, B);
impl_anonymous_message!(A, B, C);
impl_anonymous_message!(A, B, C, D);
impl_anonymous_message!(A, B, C, D, E);
impl_anonymous_message!(A, B, C, D, E, F);
impl_anonymous_message!(A, B, C, D, E, F, G);
impl_anonymous_message!(A, B, C, D, E, F, G, H);
