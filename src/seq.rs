use std::iter::Cloned;
use std::slice::Iter;
use std::str::Chars;

/// Implemented by types that have a length (including any type that implements
/// [`Seq`]) and all of the [`Op`](super::Op)s.
pub trait Len {
    /// Should return the exact length of the receiver.
    fn len(&self) -> usize;
}

/// Implemented by any sequence used as the value of a [`Delta`](super::Delta)
/// or [`Op`](super::Op).
pub trait Seq:
    for<'a> FromIterator<<Self::Iterator<'a> as Iterator>::Item> + Len + 'static
{
    /// Type of iterator over this sequence.
    type Iterator<'a>: Iterator
    where
        Self: 'a;

    /// Should return an iterator over the items in this sequence.
    fn iter(&self) -> Self::Iterator<'_>;
}

impl Len for String {
    fn len(&self) -> usize {
        self.chars().count()
    }
}

impl Seq for String {
    type Iterator<'a> = Chars<'a>;

    fn iter(&self) -> Self::Iterator<'_> {
        self.chars()
    }
}

impl<T> Len for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<T> Seq for Vec<T>
where
    T: Clone + 'static,
{
    type Iterator<'a> = Cloned<Iter<'a, T>>;

    fn iter(&self) -> Self::Iterator<'_> {
        <[T]>::iter(self).cloned()
    }
}
