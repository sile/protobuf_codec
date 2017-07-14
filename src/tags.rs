use traits::Tag;

macro_rules! impl_tag {
    ($t:ty, $n:expr) => {
        impl Tag for $t {
            fn number() -> u32 {
                $n
            }
        }
    }
}

pub struct Tag1;
impl_tag!(Tag1, 1);

pub struct Tag2;
impl_tag!(Tag2, 2);

pub struct Tag3;
impl_tag!(Tag3, 3);

pub struct Tag4;
impl_tag!(Tag4, 4);

pub struct Tag5;
impl_tag!(Tag5, 5);

pub struct Tag6;
impl_tag!(Tag6, 6);

pub struct Tag7;
impl_tag!(Tag7, 7);

pub struct Tag8;
impl_tag!(Tag8, 8);

pub struct Tag9;
impl_tag!(Tag9, 9);

pub struct Tag10;
impl_tag!(Tag10, 10);

pub struct Tag11;
impl_tag!(Tag11, 11);

pub struct Tag12;
impl_tag!(Tag12, 12);

pub struct Tag13;
impl_tag!(Tag13, 13);

pub struct Tag14;
impl_tag!(Tag14, 14);

pub struct Tag15;
impl_tag!(Tag15, 15);

pub struct Tag16;
impl_tag!(Tag16, 16);

pub struct Tag17;
impl_tag!(Tag17, 17);

pub struct Tag18;
impl_tag!(Tag18, 18);

pub struct Tag19;
impl_tag!(Tag19, 19);

pub struct Tag20;
impl_tag!(Tag20, 20);
