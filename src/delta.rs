use arbitrary::Arbitrary;
use serde::{Deserialize, Serialize};

use super::ops::{Delete, Insert, Retain};
use super::{Iter, Len, Op, Seq};

/// Series of insert, retain and delete operations.
#[derive(Arbitrary, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delta<T, A> {
    ops: Vec<Op<T, A>>,
}

impl<T, A> Delta<T, A>
where
    T: Default + Clone + Seq + Extend<T>,
    A: Clone + PartialEq,
{
    /// Returns a new empty delta (i.e. an empty series of operations).
    pub fn new() -> Delta<T, A> {
        Delta {
            ops: Default::default(),
        }
    }

    /// Returns a new delta with one insert operation with the given value and
    /// optional attributes. Pass `None` if you don't want this operation to
    /// affect any attributes. See [`Insert::attributes`] for more information.
    pub fn insert(mut self, insert: T, attributes: impl Into<Option<A>>) -> Self {
        self.push(Op::Insert(Insert {
            insert,
            attributes: attributes.into(),
        }));

        self
    }

    /// Returns a new delta that retains the given number of elements,
    /// optionally replacing their attributes with the given value. Pass `None`
    /// if you don't want this operation to affect any attributes. See
    /// [`Retain::attributes`] for more information.
    pub fn retain(mut self, retain: usize, attributes: impl Into<Option<A>>) -> Self {
        self.push(Op::Retain(Retain {
            retain,
            attributes: attributes.into(),
        }));

        self
    }

    /// Returns a new delta that deletes the given number of elements.
    pub fn delete(mut self, delete: usize) -> Self {
        self.push(Op::Delete(Delete { delete }));
        self
    }

    pub(crate) fn ops(&self) -> impl Iterator<Item = &Op<T, A>> {
        <[_]>::iter(&self.ops)
    }

    pub(crate) fn chop(mut self) -> Self {
        while let Some(Op::Retain(Retain { attributes, .. })) = self.ops.last() {
            if attributes.is_some() {
                break;
            }

            self.ops.pop();
        }

        self
    }

    /// Appends the given operation to this series. If possible, this function
    /// attempts to merge the last operation to the newly added operation.
    ///
    /// Keep in mind that this is different from [`Delta::compose`][1]. For
    /// example, pushing a [`Delete`] to a [`Delta`] will literally just add
    /// that operation to the sequence (as opposed to applying it).
    ///
    /// [1]: #impl-Compose<Delta<T,+A>>-for-Delta<T,+A>
    pub fn push(&mut self, op: Op<T, A>) {
        if op.len() == 0 {
            return;
        }

        let Some(last_op) = self.ops.last_mut() else {
            self.ops.push(op);
            return;
        };

        match last_op {
            Op::Insert(Insert {
                insert: last_insert,
                attributes: last_attributes,
            }) => match op {
                Op::Insert(Insert {
                    insert,
                    ref attributes,
                }) if last_attributes == attributes => {
                    last_insert.extend([insert]);
                }
                Op::Insert { .. } | Op::Retain { .. } | Op::Delete { .. } => {
                    self.ops.push(op);
                }
            },
            Op::Retain(Retain {
                retain: last_retain,
                attributes: last_attributes,
            }) => match op {
                Op::Retain(Retain { retain, attributes }) if last_attributes == &attributes => {
                    match last_retain.overflowing_add(retain) {
                        (retain, false) => *last_retain = retain,
                        (retain, true) => {
                            *last_retain = usize::MAX;
                            self.ops.push(Op::Retain(Retain {
                                retain: retain + 1,
                                attributes,
                            }))
                        }
                    }
                }
                Op::Insert { .. } | Op::Retain { .. } | Op::Delete { .. } => {
                    self.ops.push(op);
                }
            },
            Op::Delete(Delete {
                delete: last_delete,
            }) => match op {
                Op::Insert { .. } => {
                    if let Some(delete) = self.ops.pop() {
                        self.push(op);
                        self.push(delete);
                    }
                }
                Op::Retain { .. } => {
                    self.ops.push(op);
                }
                Op::Delete(Delete { delete }) => match last_delete.overflowing_add(delete) {
                    (delete, false) => *last_delete = delete,
                    (delete, true) => {
                        *last_delete = usize::MAX;
                        self.ops.push(Op::Delete(Delete { delete: delete + 1 }))
                    }
                },
            },
        }
    }
}

impl<T, A> Extend<Op<T, A>> for Delta<T, A>
where
    T: Clone + Default + Seq + Extend<T>,
    A: Clone + Default + PartialEq,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Op<T, A>>,
    {
        iter.into_iter().for_each(|op| self.push(op))
    }
}

impl<T, A> FromIterator<Op<T, A>> for Delta<T, A>
where
    T: Clone + Default + Seq + Extend<T>,
    A: Clone + Default + PartialEq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Op<T, A>>,
    {
        let mut delta = Delta::new();
        delta.extend(iter);
        delta
    }
}

impl<T, A> IntoIterator for Delta<T, A>
where
    T: Clone + Default + Seq,
    A: Clone + Default,
{
    type Item = Op<T, A>;

    type IntoIter = Iter<T, A>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.ops.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::{Delete, Delta, Insert, Op, Retain};

    #[test]
    fn test_push_insert_insert_same() {
        let mut iter = Delta::<_, ()>::new()
            .insert("a".to_owned(), None)
            .insert("b".to_owned(), None)
            .into_iter();

        assert_eq!(
            iter.next(),
            Some(Op::Insert(Insert {
                insert: "ab".to_owned(),
                attributes: None
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_push_insert_insert_diff() {
        let mut iter = Delta::new()
            .insert("a".to_owned(), false)
            .insert("b".to_owned(), true)
            .into_iter();

        assert_eq!(
            iter.next(),
            Some(Op::Insert(Insert {
                insert: "a".to_owned(),
                attributes: Some(false)
            }))
        );
        assert_eq!(
            iter.next(),
            Some(Op::Insert(Insert {
                insert: "b".to_owned(),
                attributes: Some(true)
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_push_insert_retain() {
        let mut iter = Delta::new()
            .insert("a".to_owned(), ())
            .retain(1, Some(()))
            .into_iter();

        assert_eq!(
            iter.next(),
            Some(Op::Insert(Insert {
                insert: "a".to_owned(),
                attributes: Some(())
            }))
        );
        assert_eq!(
            iter.next(),
            Some(Op::Retain(Retain {
                retain: 1,
                attributes: Some(())
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_push_insert_delete() {
        let mut iter = Delta::<_, ()>::new()
            .insert("a".to_owned(), None)
            .delete(1)
            .into_iter();

        assert_eq!(
            iter.next(),
            Some(Op::Insert(Insert {
                insert: "a".to_owned(),
                attributes: None
            }))
        );
        assert_eq!(iter.next(), Some(Op::Delete(Delete { delete: 1 })));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_push_retain_insert() {
        let mut iter = Delta::<_, ()>::new()
            .retain(1, None)
            .insert("a".to_owned(), None)
            .into_iter();

        assert_eq!(
            iter.next(),
            Some(Op::Retain(Retain {
                retain: 1,
                attributes: None
            }))
        );
        assert_eq!(
            iter.next(),
            Some(Op::Insert(Insert {
                insert: "a".to_owned(),
                attributes: None
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_push_retain_retain_same() {
        let mut iter = Delta::<String, ()>::new()
            .retain(1, None)
            .retain(2, None)
            .into_iter();

        assert_eq!(
            iter.next(),
            Some(Op::Retain(Retain {
                retain: 3,
                attributes: None
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_push_retain_retain_diff() {
        let mut iter = Delta::<String, _>::new()
            .retain(1, false)
            .retain(2, true)
            .into_iter();

        assert_eq!(
            iter.next(),
            Some(Op::Retain(Retain {
                retain: 1,
                attributes: Some(false)
            }))
        );
        assert_eq!(
            iter.next(),
            Some(Op::Retain(Retain {
                retain: 2,
                attributes: Some(true)
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_push_retain_delete() {
        let mut iter = Delta::<String, ()>::new()
            .retain(1, None)
            .delete(1)
            .into_iter();

        assert_eq!(
            iter.next(),
            Some(Op::Retain(Retain {
                retain: 1,
                attributes: None
            }))
        );
        assert_eq!(iter.next(), Some(Op::Delete(Delete { delete: 1 })));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_push_delete_insert() {
        let mut iter = Delta::<_, ()>::new()
            .delete(1)
            .insert("a".to_owned(), None)
            .into_iter();

        assert_eq!(
            iter.next(),
            Some(Op::Insert(Insert {
                insert: "a".to_owned(),
                attributes: None
            }))
        );
        assert_eq!(iter.next(), Some(Op::Delete(Delete { delete: 1 })));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_push_delete_retain() {
        let mut iter = Delta::<String, ()>::new()
            .delete(1)
            .retain(1, None)
            .into_iter();

        assert_eq!(iter.next(), Some(Op::Delete(Delete { delete: 1 })));
        assert_eq!(
            iter.next(),
            Some(Op::Retain(Retain {
                retain: 1,
                attributes: None
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_push_delete_delete() {
        let mut iter = Delta::<String, ()>::new().delete(1).delete(1).into_iter();

        assert_eq!(iter.next(), Some(Op::Delete(Delete { delete: 2 })));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_overflow() {
        let mut x = Delta::<String, ()>::new();
        x.push(Op::Retain(Retain {
            retain: usize::MAX - 4,
            attributes: None,
        }));
        x.push(Op::Retain(Retain {
            retain: 8,
            attributes: None,
        }));

        let mut iter = x.into_iter();

        assert_eq!(
            iter.next().unwrap(),
            Op::Retain(Retain {
                retain: usize::MAX,
                attributes: None
            })
        );

        assert_eq!(
            iter.next().unwrap(),
            Op::Retain(Retain {
                retain: 4,
                attributes: None
            })
        );
    }
}
