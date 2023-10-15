use std::{iter::from_fn, vec::IntoIter};

use super::{Len, Op, Seq};

/// Iterator over [`Ops`](Op) with a utility function to zip two iters together
/// and apply a map function that supports partial consumption of either op, as
/// used by [`Compose`](crate::Compose) and [`Transform`](crate::Transform).
pub struct Iter<T, A> {
    iter: IntoIter<Op<T, A>>,
    partial: Option<Op<T, A>>,
}

impl<T, A> Iter<T, A>
where
    T: Clone + Default + Seq,
    A: Clone + Default,
{
    pub(crate) fn new(iter: IntoIter<Op<T, A>>) -> Iter<T, A> {
        Iter {
            iter,
            partial: Default::default(),
        }
    }

    /// Returns a mutable reference to the next op. The caller of this function
    /// may use the mutable reference to partially consume the next op (instead
    /// of fully). For example, this is used by [`Compose`](crate::Compose) and
    /// [`Transform`](crate::Transform) to process both next items in place.
    pub fn next_mut(&mut self) -> Option<&mut Op<T, A>> {
        match &self.partial {
            Some(partial) if partial.len() > 0 => self.partial.as_mut(),
            Some(_) | None => {
                self.partial = self.iter.next();
                self.partial.as_mut()
            }
        }
    }

    /// Utility function that zips two iters and applies the given map function
    /// to each pair of ops. This function may choose to only partially consume
    /// an op. The remainder of that op will be fed to the next invocation. This
    /// will continue until either iterator is exhausted. Note that this means
    /// that the iterators are not necessarily both exhausted when this function
    /// returns.
    pub fn zip_mut<'a, F, U>(
        &'a mut self,
        other: &'a mut Iter<T, A>,
        map_fn: F,
    ) -> impl Iterator<Item = U> + 'a
    where
        F: for<'b> Fn(&'b mut Op<T, A>, &'b mut Op<T, A>) -> U + 'a,
    {
        from_fn(move || match (self.next_mut(), other.next_mut()) {
            (Some(self_op), Some(other_op)) => Some(map_fn(self_op, other_op)),
            _ => None,
        })
    }
}

impl<T, A> Iterator for Iter<T, A>
where
    T: Default + Seq,
    A: Default,
{
    type Item = Op<T, A>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.partial.take() {
            Some(partial) if partial.len() > 0 => Some(partial),
            Some(_) | None => self.iter.next(),
        }
    }
}
