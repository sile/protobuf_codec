use traits;

macro_rules! impl_tag {
    ($t:ty, $n:expr) => {
        impl traits::Tag for $t {
            fn number() -> u32 {
                $n
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Tag1;
impl_tag!(Tag1, 1);

#[derive(Debug, Default)]
pub struct Tag2;
impl_tag!(Tag2, 2);

#[derive(Debug, Default)]
pub struct Tag3;
impl_tag!(Tag3, 3);

#[derive(Debug, Default)]
pub struct Tag4;
impl_tag!(Tag4, 4);

#[derive(Debug, Default)]
pub struct Tag5;
impl_tag!(Tag5, 5);

#[derive(Debug, Default)]
pub struct Tag6;
impl_tag!(Tag6, 6);

#[derive(Debug, Default)]
pub struct Tag7;
impl_tag!(Tag7, 7);

#[derive(Debug, Default)]
pub struct Tag8;
impl_tag!(Tag8, 8);

#[derive(Debug, Default)]
pub struct Tag9;
impl_tag!(Tag9, 9);

#[derive(Debug, Default)]
pub struct Tag10;
impl_tag!(Tag10, 10);

#[derive(Debug, Default)]
pub struct Tag11;
impl_tag!(Tag11, 11);

#[derive(Debug, Default)]
pub struct Tag12;
impl_tag!(Tag12, 12);

#[derive(Debug, Default)]
pub struct Tag13;
impl_tag!(Tag13, 13);

#[derive(Debug, Default)]
pub struct Tag14;
impl_tag!(Tag14, 14);

#[derive(Debug, Default)]
pub struct Tag15;
impl_tag!(Tag15, 15);
