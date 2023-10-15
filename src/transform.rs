use std::cmp::min;
use std::fmt::Debug;
use std::mem::take;

use super::op::split;
use super::ops::{Delete, Insert, Retain};
use super::{Delta, Len, Op, Seq};

/// Implemented by types that can transform another operation to make them
/// behave commutatively (i.e. order-independent).
///
/// The table below contains links to the implementation notes of [`Transform`]
/// for each pair of [`Op`]. The rows represent Alice's operations (i.e. the
/// first) and the columns represent Bob's operations (i.e. the second), e.g.
/// bottom left means Alice's delete followed by Bob's insert.
///
/// | ↱      | Insert    | Retain    | Delete    |
/// |:-------|:----------|:----------|:----------|
/// | Insert | [Impl][1] | [Impl][4] | [Impl][7] |
/// | Retain | [Impl][2] | [Impl][5] | [Impl][8] |
/// | Delete | [Impl][3] | [Impl][6] | [Impl][9] |
///
/// [1]: #impl-Transform<%26mut+Insert<T,+A>>-for-%26mut+Insert<T,+A>
/// [2]: #impl-Transform<%26mut+Insert<T,+A>>-for-%26mut+Retain<A>
/// [3]: #impl-Transform<%26mut+Insert<T,+A>>-for-%26mut+Delete
/// [4]: #impl-Transform<%26mut+Retain<A>>-for-%26mut+Insert<T,+A>
/// [5]: #impl-Transform<%26mut+Retain<A>>-for-%26mut+Retain<A>
/// [6]: #impl-Transform<%26mut+Retain<A>>-for-%26mut+Delete
/// [7]: #impl-Transform<%26mut+Delete>-for-%26mut+Insert<T,+A>
/// [8]: #impl-Transform<%26mut+Delete>-for-%26mut+Retain<A>
/// [9]: #impl-Transform<%26mut+Delete>-for-%26mut+Delete
pub trait Transform<Rhs> {
    /// Output type that transforming another value with the receiver produces.
    type Output;

    /// Transforms the given value with the receiver.
    fn transform(self, rhs: Rhs, priority: bool) -> Self::Output;
}

/// Alice and Bob are both inserting at the same position. Both want their text
/// to be preserved. If Alice has `priority`, Alice's text will be inserted
/// first, so Bob has to retain before his own insert. If Alice hasn't
/// `priority`, Bob's text will be inserted first, so Bob has to retain after
/// his own insert.
impl<T, A> Transform<&mut Insert<T, A>> for &mut Insert<T, A>
where
    T: Clone + Default + Extend<T> + Seq,
    A: Clone + Default + PartialEq,
{
    type Output = Op<T, A>;

    fn transform(self, rhs: &mut Insert<T, A>, priority: bool) -> Self::Output {
        match priority {
            true => take(self).as_retain().into(),
            false => take(rhs).into(),
        }
    }
}

/// Alice is inserting and Bob is retaining. Bob's position is after Alice's
/// position. Alice's text will always be inserted first, so Bob has to increase
/// his retain.
impl<T, A> Transform<&mut Retain<A>> for &mut Insert<T, A>
where
    T: Clone + Default + Extend<T> + Seq,
    A: Clone + Default + PartialEq,
{
    type Output = Retain<A>;

    fn transform(self, _rhs: &mut Retain<A>, _priority: bool) -> Self::Output {
        take(self).as_retain().into()
    }
}

/// Alice is inserting while Bob is deleting. Bob is unaware of Alice insert, so
/// it's unlikely he intended to undo her insert.  Therefore, Alice's insert has
/// precedence and Bob will need to retain before his delete.
impl<T, A> Transform<&mut Delete> for &mut Insert<T, A>
where
    T: Default + Len,
    A: Default,
{
    type Output = Retain<A>;

    fn transform(self, _rhs: &mut Delete, _priority: bool) -> Self::Output {
        take(self).as_retain()
    }
}

/// Alice is retaining while Bob is inserting. Bob's text will always be
/// inserted first.
impl<T, A> Transform<&mut Insert<T, A>> for &mut Retain<A>
where
    T: Clone + Default + Seq + Extend<T>,
    A: Clone + Default + PartialEq,
{
    type Output = Insert<T, A>;

    fn transform(self, rhs: &mut Insert<T, A>, _priority: bool) -> Self::Output {
        take(rhs)
    }
}

/// Alice and Bob are both retaining the same selection. We can simply retain
/// Bob's retain.
impl<A> Transform<&mut Retain<A>> for &mut Retain<A>
where
    A: Clone + Default,
{
    type Output = Retain<A>;

    fn transform(self, rhs: &mut Retain<A>, priority: bool) -> Self::Output {
        let (lhs, rhs) = split(self, rhs);

        match priority {
            true => lhs.or(rhs),
            false => rhs.or(lhs),
        }
    }
}

/// Alice is retaining the selection that Bob deleted.
impl<A> Transform<&mut Delete> for &mut Retain<A>
where
    A: Clone + Default,
{
    type Output = Delete;

    fn transform(self, rhs: &mut Delete, _priority: bool) -> Self::Output {
        let (_lhs, rhs) = split(self, rhs);

        rhs
    }
}

/// Alice is deleting while Bob is inserting. Bob's text will always be inserted
/// first.
impl<T, A> Transform<&mut Insert<T, A>> for &mut Delete
where
    T: Default,
    A: Default,
{
    type Output = Insert<T, A>;

    fn transform(self, rhs: &mut Insert<T, A>, _priority: bool) -> Self::Output {
        take(rhs)
    }
}

/// Alice is deleting the same selection that Bob is retaining. Since Alice's
/// delete has already been applied, we ignore Bob's retain.
impl<A> Transform<&mut Retain<A>> for &mut Delete
where
    A: Clone + Default,
{
    type Output = Delete;

    fn transform(self, rhs: &mut Retain<A>, _priority: bool) -> Self::Output {
        let (_, _) = split(self, rhs);

        Default::default()
    }
}

/// Alice is deleting the same selection as Bob. Since Alice's delete has
/// already been applied, we ignore Bob's delete.
impl Transform<&mut Delete> for &mut Delete {
    type Output = Delete;

    fn transform(self, rhs: &mut Delete, _priority: bool) -> Self::Output {
        let (_, _) = split(self, rhs);

        Default::default()
    }
}

impl<T, A> Transform<&mut Op<T, A>> for &mut Op<T, A>
where
    T: Clone + Default + Seq + Extend<T>,
    A: Clone + Default + PartialEq,
{
    type Output = Op<T, A>;

    fn transform(self, rhs: &mut Op<T, A>, priority: bool) -> Self::Output {
        match self {
            Op::Insert(lhs) => match rhs {
                Op::Insert(rhs) => lhs.transform(rhs, priority).into(),
                Op::Retain(rhs) => lhs.transform(rhs, priority).into(),
                Op::Delete(rhs) => lhs.transform(rhs, priority).into(),
            },
            Op::Retain(lhs) => match rhs {
                Op::Insert(rhs) => lhs.transform(rhs, priority).into(),
                Op::Retain(rhs) => lhs.transform(rhs, priority).into(),
                Op::Delete(rhs) => lhs.transform(rhs, priority).into(),
            },
            Op::Delete(lhs) => match rhs {
                Op::Insert(rhs) => lhs.transform(rhs, priority).into(),
                Op::Retain(rhs) => lhs.transform(rhs, priority).into(),
                Op::Delete(rhs) => lhs.transform(rhs, priority).into(),
            },
        }
    }
}

impl<T, A> Transform<Delta<T, A>> for Delta<T, A>
where
    T: Clone + Default + Seq + Extend<T> + Debug,
    A: Clone + Default + PartialEq + Debug,
{
    type Output = Delta<T, A>;

    fn transform(self, rhs: Delta<T, A>, priority: bool) -> Self::Output {
        let mut self_iter = self.into_iter();
        let mut other_iter = rhs.into_iter();

        let mut result = Delta::new();

        result.extend(self_iter.zip_mut(&mut other_iter, |a, b| a.transform(b, priority)));
        result.extend(other_iter);

        result.chop()
    }
}

impl<T, A> Transform<usize> for &Delta<T, A>
where
    T: Clone + Default + Seq + Extend<T>,
    A: Clone + Default + PartialEq,
{
    type Output = usize;

    fn transform(self, rhs: usize, priority: bool) -> Self::Output {
        let mut index = rhs;
        let mut offset = 0;
        let mut iter = self.ops();

        while let Some(op) = iter.next() {
            if offset > rhs {
                break;
            }

            match op {
                Op::Insert(insert) => {
                    if offset < index || !priority {
                        index += insert.len()
                    }

                    offset += insert.len()
                }
                Op::Retain(retain) => {
                    offset += retain.len();
                }
                Op::Delete(delete) => {
                    index -= min(delete.len(), index - offset);
                }
            }
        }

        index
    }
}

#[cfg(test)]
mod test {
    use super::{Delta, Transform};

    #[test]
    fn test_insert_before_position() {
        let delta = Delta::new().insert("A".to_owned(), ());

        assert_eq!((&delta).transform(2, true), 3);
        assert_eq!((&delta).transform(2, false), 3);
    }

    #[test]
    fn test_insert_after_position() {
        let delta = Delta::new().retain(2, ()).insert("A".to_owned(), ());

        assert_eq!((&delta).transform(1, true), 1);
        assert_eq!((&delta).transform(1, false), 1);
    }

    #[test]
    fn test_insert_at_position() {
        let delta = Delta::new().retain(2, ()).insert("A".to_owned(), ());

        assert_eq!((&delta).transform(2, true), 2);
        assert_eq!((&delta).transform(2, false), 3);
    }
}
