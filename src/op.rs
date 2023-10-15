use std::cmp::min;

use arbitrary::Arbitrary;
use serde::{Deserialize, Serialize};

use super::ops::{Delete, Insert, Retain};
use super::{Len, Seq};

/// Implemented by types that can split their value in two at any given index.
pub trait Split {
    /// Removes and returns the first first `len` items from this sequence.
    fn split(&mut self, len: usize) -> Self;
}

pub fn split<T, U>(lhs: &mut T, rhs: &mut U) -> (T, U)
where
    T: Len + Split,
    U: Len + Split,
{
    let len = min(lhs.len(), rhs.len());

    (lhs.split(len), rhs.split(len))
}

/// Individual insert, retain or delete operation.
#[derive(Arbitrary, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Op<T, A = ()> {
    /// Represents an insert-operation with a value and optional attributes.
    /// [Click here](Insert) to read more about insert operations.
    Insert(Insert<T, A>),

    /// Represents a retain-operation with a length and optional attributes.
    /// [Click here](Retain) to read more about retain operations.
    Retain(Retain<A>),

    /// Represents a delete-operation with a length. [Click here](Delete) to
    /// read more about retain operations.
    Delete(Delete),
}

impl<T, A> Len for Op<T, A>
where
    T: Len,
{
    fn len(&self) -> usize {
        match self {
            Self::Insert(insert) => insert.len(),
            Self::Retain(retain) => retain.len(),
            Self::Delete(delete) => delete.len(),
        }
    }
}

impl<T, A> Default for Op<T, A> {
    fn default() -> Self {
        Op::Delete(Delete { delete: 0 })
    }
}

impl<T, A> From<Insert<T, A>> for Op<T, A> {
    fn from(value: Insert<T, A>) -> Self {
        Self::Insert(value)
    }
}

impl<T, A> From<Retain<A>> for Op<T, A> {
    fn from(value: Retain<A>) -> Self {
        Self::Retain(value)
    }
}

impl<T, A> From<Delete> for Op<T, A> {
    fn from(value: Delete) -> Self {
        Self::Delete(value)
    }
}

impl<T, A> Split for Op<T, A>
where
    T: Clone + Seq,
    A: Clone,
{
    fn split(&mut self, len: usize) -> Op<T, A> {
        let len = min(self.len(), len);

        match self {
            Self::Insert(insert) => insert.split(len).into(),
            Self::Retain(retain) => retain.split(len).into(),
            Self::Delete(delete) => delete.split(len).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Delete, Insert, Op, Split};

    #[test]
    fn test_split_insert_start() {
        let mut a = Op::Insert(Insert {
            insert: "abc".to_owned(),
            attributes: Some(()),
        });
        let b = a.split(0);

        assert_eq!(
            a,
            Op::Insert(Insert {
                insert: "abc".to_owned(),
                attributes: Some(()),
            })
        );

        assert_eq!(
            b,
            Op::Insert(Insert {
                insert: "".to_owned(),
                attributes: Some(()),
            })
        );
    }

    #[test]
    fn test_split_insert_mid() {
        let mut a = Op::Insert(Insert {
            insert: "abc".to_owned(),
            attributes: Some(()),
        });
        let b = a.split(1);

        assert_eq!(
            a,
            Op::Insert(Insert {
                insert: "bc".to_owned(),
                attributes: Some(()),
            })
        );

        assert_eq!(
            b,
            Op::Insert(Insert {
                insert: "a".to_owned(),
                attributes: Some(()),
            })
        );
    }

    #[test]
    fn test_split_insert_end() {
        let mut a = Op::Insert(Insert {
            insert: "abc".to_owned(),
            attributes: Some(()),
        });
        let b = a.split(3);

        assert_eq!(
            a,
            Op::Insert(Insert {
                insert: "".to_owned(),
                attributes: Some(()),
            })
        );

        assert_eq!(
            b,
            Op::Insert(Insert {
                insert: "abc".to_owned(),
                attributes: Some(()),
            })
        );
    }

    #[test]
    fn test_split_insert_oob() {
        let mut a = Op::Insert(Insert {
            insert: "abc".to_owned(),
            attributes: Some(()),
        });
        let b = a.split(4);

        assert_eq!(
            a,
            Op::Insert(Insert {
                insert: "".to_owned(),
                attributes: Some(()),
            })
        );

        assert_eq!(
            b,
            Op::Insert(Insert {
                insert: "abc".to_owned(),
                attributes: Some(()),
            })
        );
    }

    #[test]
    fn test_split_delete_start() {
        let mut a = Op::<String, ()>::Delete(Delete { delete: 3 });
        let b = a.split(0);

        assert_eq!(a, Op::Delete(Delete { delete: 3 }));
        assert_eq!(b, Op::Delete(Delete { delete: 0 }));
    }

    #[test]
    fn test_split_delete_mid() {
        let mut a = Op::<String, ()>::Delete(Delete { delete: 3 });
        let b = a.split(1);

        assert_eq!(a, Op::Delete(Delete { delete: 2 }));
        assert_eq!(b, Op::Delete(Delete { delete: 1 }));
    }

    #[test]
    fn test_split_delete_end() {
        let mut a = Op::<String, ()>::Delete(Delete { delete: 3 });
        let b = a.split(3);

        assert_eq!(a, Op::Delete(Delete { delete: 0 }));
        assert_eq!(b, Op::Delete(Delete { delete: 3 }));
    }
}
