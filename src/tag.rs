/// Field tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tag(pub u32);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tag1;
impl From<Tag1> for Tag {
    fn from(_: Tag1) -> Self {
        Tag(1)
    }
}
impl PartialEq<Tag> for Tag1 {
    fn eq(&self, other: &Tag) -> bool {
        other.0 == 1
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tag2;
impl From<Tag2> for Tag {
    fn from(_: Tag2) -> Self {
        Tag(2)
    }
}
impl PartialEq<Tag> for Tag2 {
    fn eq(&self, other: &Tag) -> bool {
        other.0 == 2
    }
}
