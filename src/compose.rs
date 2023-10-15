use std::fmt::Debug;
use std::mem::take;

use arbitrary::Arbitrary;

use super::op::split;
use super::ops::{Delete, Insert, Retain};
use super::{Delta, Op, Seq};

/// Implemented by types that can apply a series of operations in sequence.
///
/// The table below contains links to the implementation notes of [`Compose`]
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
/// [1]: #impl-Compose<%26mut+Insert<T,+A>>-for-U
/// [2]: #impl-Compose<%26mut+Insert<T,+A>>-for-U
/// [3]: #impl-Compose<%26mut+Insert<T,+A>>-for-U
/// [4]: #impl-Compose<%26mut+Retain<A>>-for-%26mut+Insert<T,+A>
/// [5]: #impl-Compose<%26mut+Retain<A>>-for-%26mut+Retain<A>
/// [6]: #impl-Compose<%26mut+Retain<A>>-for-%26mut+Delete
/// [7]: #impl-Compose<%26mut+Delete>-for-%26mut+Insert<T,+A>
/// [8]: #impl-Compose<%26mut+Delete>-for-%26mut+Retain<A>
/// [9]: #impl-Compose<%26mut+Delete>-for-%26mut+Delete
pub trait Compose<Rhs> {
    /// Output type that applying a series of operations to this type produces.
    type Output;

    /// Applies the given series of operations to the receiver and returns the
    /// result.
    fn compose(self, rhs: Rhs) -> Self::Output;
}

#[doc(hidden)]
#[derive(Arbitrary, Clone, Debug, Default, PartialEq, Eq)]
pub struct LastWriteWins<T>(pub T);

impl<T> Compose<LastWriteWins<T>> for LastWriteWins<T> {
    type Output = LastWriteWins<T>;

    fn compose(self, rhs: LastWriteWins<T>) -> Self::Output {
        rhs
    }
}

impl Compose<()> for () {
    type Output = ();

    fn compose(self, _rhs: ()) -> Self::Output {
        self
    }
}

impl<T> Compose<Option<T>> for Option<T>
where
    T: Compose<T, Output = T>,
{
    type Output = Option<T>;

    fn compose(self, rhs: Option<T>) -> Self::Output {
        match (self, rhs) {
            (Some(lhs), Some(rhs)) => Some(lhs.compose(rhs)),
            (Some(lhs), None) => Some(lhs),
            (None, Some(rhs)) => Some(rhs),
            (None, None) => None,
        }
    }
}

impl<T, A> Compose<&mut Retain<A>> for &mut Insert<T, A>
where
    T: Seq,
    A: Clone + Compose<A, Output = A>,
{
    type Output = Insert<T, A>;

    fn compose(self, rhs: &mut Retain<A>) -> Self::Output {
        let (lhs, rhs) = split(self, rhs);

        Insert {
            insert: lhs.insert,
            attributes: lhs.attributes.compose(rhs.attributes),
        }
    }
}

impl<T, A> Compose<&mut Delete> for &mut Insert<T, A>
where
    T: Seq,
    A: Clone,
{
    type Output = Delete;

    fn compose(self, rhs: &mut Delete) -> Self::Output {
        let (_, _) = split(self, rhs);

        Default::default()
    }
}

impl<A> Compose<&mut Retain<A>> for &mut Retain<A>
where
    A: Clone + Compose<A, Output = A>,
{
    type Output = Retain<A>;

    fn compose(self, rhs: &mut Retain<A>) -> Self::Output {
        let (lhs, rhs) = split(self, rhs);

        Retain {
            retain: lhs.retain,
            attributes: lhs.attributes.compose(rhs.attributes),
        }
    }
}

impl<A> Compose<&mut Delete> for &mut Retain<A>
where
    A: Clone,
{
    type Output = Delete;

    fn compose(self, rhs: &mut Delete) -> Self::Output {
        let (_lhs, rhs) = split(self, rhs);

        rhs.into()
    }
}

impl<A, T, U> Compose<&mut Insert<T, A>> for U
where
    T: Default,
    A: Default,
{
    type Output = Insert<T, A>;

    fn compose(self, rhs: &mut Insert<T, A>) -> Self::Output {
        take(rhs)
    }
}

impl<A> Compose<&mut Retain<A>> for &mut Delete {
    type Output = Delete;

    fn compose(self, _rhs: &mut Retain<A>) -> Self::Output {
        take(self)
    }
}

impl Compose<&mut Delete> for &mut Delete {
    type Output = Delete;

    fn compose(self, _rhs: &mut Delete) -> Self::Output {
        take(self)
    }
}

impl<T, A> Compose<&mut Op<T, A>> for &mut Op<T, A>
where
    T: Default + Clone + Seq + Extend<T>,
    A: Default + Clone + PartialEq + Compose<A, Output = A>,
{
    type Output = Op<T, A>;

    fn compose(self, rhs: &mut Op<T, A>) -> Self::Output {
        match self {
            Op::Insert(lhs) => match rhs {
                Op::Insert(rhs) => lhs.compose(rhs).into(),
                Op::Retain(rhs) => lhs.compose(rhs).into(),
                Op::Delete(rhs) => lhs.compose(rhs).into(),
            },
            Op::Retain(lhs) => match rhs {
                Op::Insert(rhs) => lhs.compose(rhs).into(),
                Op::Retain(rhs) => lhs.compose(rhs).into(),
                Op::Delete(rhs) => lhs.compose(rhs).into(),
            },
            Op::Delete(lhs) => match rhs {
                Op::Insert(rhs) => lhs.compose(rhs).into(),
                Op::Retain(rhs) => lhs.compose(rhs).into(),
                Op::Delete(rhs) => lhs.compose(rhs).into(),
            },
        }
    }
}

impl<T, A> Compose<Delta<T, A>> for Delta<T, A>
where
    T: Default + Clone + Seq + Extend<T> + Debug,
    A: Default + Clone + PartialEq + Debug + Compose<A, Output = A>,
{
    type Output = Self;

    fn compose(self, rhs: Delta<T, A>) -> Self {
        let mut self_iter = self.into_iter();
        let mut other_iter = rhs.into_iter();

        let mut result = Delta::new();

        result.extend(self_iter.zip_mut(&mut other_iter, |a, b| a.compose(b)));
        result.extend(self_iter.chain(other_iter));

        result.chop()
    }
}

#[cfg(test)]
mod tests {
    use super::{Compose, Delta};

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct Attributes {
        bold: Option<bool>,
    }

    impl Attributes {
        pub fn bold() -> Attributes {
            Attributes { bold: Some(true) }
        }
    }

    impl Compose<Attributes> for Attributes {
        type Output = Attributes;

        fn compose(self, rhs: Attributes) -> Self::Output {
            Attributes {
                bold: rhs.bold.or(self.bold),
            }
        }
    }

    #[test]
    fn test_insert_insert() {
        let a = Delta::new().insert("A".to_owned(), ());
        let b = Delta::new().insert("B".to_owned(), ());

        assert_eq!(a.compose(b), Delta::new().insert("BA".to_owned(), ()));
    }

    #[test]
    fn test_insert_retain() {
        let a = Delta::new().insert("A".to_owned(), ());
        let b = Delta::new().retain(1, ());

        assert_eq!(a.compose(b), Delta::new().insert("A".to_owned(), ()));
    }

    #[test]
    fn test_insert_delete() {
        let a = Delta::new().insert("A".to_owned(), ());
        let b = Delta::new().delete(1);

        assert_eq!(a.compose(b), Delta::new());
    }

    #[test]
    fn test_retain_insert() {
        let a = Delta::new().retain(1, Attributes::bold());
        let b = Delta::new().insert("A".to_owned(), None);

        assert_eq!(
            a.compose(b),
            Delta::new()
                .insert("A".to_owned(), None)
                .retain(1, Attributes::bold())
        );
    }

    #[test]
    fn test_retain_retain() {
        let a = Delta::<String, _>::new().retain(1, None);
        let b = Delta::new().retain(2, Attributes::bold());

        assert_eq!(a.compose(b), Delta::new().retain(2, Attributes::bold()));
    }

    #[test]
    fn test_retain_delete() {
        let a = Delta::<String, _>::new().retain(1, ());
        let b = Delta::new().delete(1);

        assert_eq!(a.compose(b), Delta::new().delete(1));
    }

    #[test]
    fn test_delete_insert() {
        let a = Delta::new().delete(1);
        let b = Delta::new().insert("B".to_owned(), ());

        assert_eq!(
            a.compose(b),
            Delta::new().insert("B".to_owned(), ()).delete(1)
        );
    }

    #[test]
    fn test_delete_retain() {
        let a = Delta::<String, _>::new().delete(1);
        let b = Delta::new().retain(1, Attributes::bold());

        assert_eq!(
            a.compose(b),
            Delta::new().delete(1).retain(1, Attributes::bold())
        );
    }

    #[test]
    fn test_delete_delete() {
        let a = Delta::<String, ()>::new().delete(1);
        let b = Delta::new().delete(2);

        assert_eq!(a.compose(b), Delta::new().delete(3));
    }

    #[test]
    fn test_insert_mid() {
        let a = Delta::new().insert("Hello".to_owned(), ());
        let b = Delta::new().retain(3, ()).insert("X".to_owned(), ());

        assert_eq!(a.compose(b), Delta::new().insert("HelXlo".to_owned(), ()));
    }

    #[test]
    fn test_delete_all() {
        let a = Delta::new().retain(4, ()).insert("Hello".to_owned(), ());
        let b = Delta::new().delete(9);

        assert_eq!(a.compose(b), Delta::new().delete(4));
    }

    #[test]
    fn test_over_retain() {
        let a = Delta::<_, ()>::new().insert("Hello".to_owned(), None);
        let b = Delta::new().retain(10, None);

        assert_eq!(a.compose(b), Delta::new().insert("Hello".to_owned(), None));
    }

    #[test]
    fn test_retain_start_optimization() {
        let a = Delta::new()
            .insert("A".to_owned(), Attributes::bold())
            .insert("B".to_owned(), None)
            .insert("C".to_owned(), Attributes::bold())
            .delete(1);
        let b = Delta::new().retain(3, None).insert("D".to_owned(), None);

        assert_eq!(
            a.compose(b),
            Delta::new()
                .insert("A".to_owned(), Attributes::bold())
                .insert("B".to_owned(), None)
                .insert("C".to_owned(), Attributes::bold())
                .insert("D".to_owned(), None)
                .delete(1)
        );
    }

    #[test]
    fn test_retain_start_optimization_split() {
        let a = Delta::new()
            .insert("A".to_owned(), Attributes::bold())
            .insert("B".to_owned(), None)
            .insert("C".to_owned(), Attributes::bold())
            .retain(5, None)
            .delete(1);
        let b = Delta::new().retain(4, None).insert("D".to_owned(), None);

        assert_eq!(
            a.compose(b),
            Delta::new()
                .insert("A".to_owned(), Attributes::bold())
                .insert("B".to_owned(), None)
                .insert("C".to_owned(), Attributes::bold())
                .retain(1, None)
                .insert("D".to_owned(), None)
                .retain(4, None)
                .delete(1)
        );
    }

    #[test]
    fn test_retain_end_optimization() {
        let a = Delta::new()
            .insert("A".to_owned(), Attributes::bold())
            .insert("B".to_owned(), None)
            .insert("C".to_owned(), Attributes::bold());
        let b = Delta::new().delete(1);

        assert_eq!(
            a.compose(b),
            Delta::new()
                .insert("B".to_owned(), None)
                .insert("C".to_owned(), Attributes::bold())
        );
    }

    #[test]
    fn test_retain_end_optimization_join() {
        let a = Delta::new()
            .insert("A".to_owned(), Attributes::bold())
            .insert("B".to_owned(), None)
            .insert("C".to_owned(), Attributes::bold())
            .insert("D".to_owned(), None)
            .insert("E".to_owned(), Attributes::bold())
            .insert("F".to_owned(), None);
        let b = Delta::new().retain(1, None).delete(1);

        assert_eq!(
            a.compose(b),
            Delta::new()
                .insert("AC".to_owned(), Attributes::bold())
                .insert("D".to_owned(), None)
                .insert("E".to_owned(), Attributes::bold())
                .insert("F".to_owned(), None)
        );
    }
}
