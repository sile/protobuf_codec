// The smallest tag number you can specify is 1,
// and the largest is 229 - 1, or 536,870,911.
// You also cannot use the numbers 19000 through 19999
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
