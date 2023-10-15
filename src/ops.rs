//! Types that represent the insert, retain and delete operations within Kyte.

use arbitrary::Arbitrary;
use serde::{Deserialize, Serialize};

use super::{Len, Seq, Split};

/// Represents an operation that inserts a sequence with optional attributes.
///
/// For more information about the compositions and transformations of this
/// operation, read the documentation on the following trait implementations:
///
/// | Alice  | Bob    | Composition | Transformation |
/// |:-------|:-------|:------------|:---------------|
/// | Insert | Insert | [`Compose<&mut Insert>`][1] | [`Transform<&mut Insert>`][2] |
/// | Insert | Retain | [`Compose<&mut Retain>`][3] | [`Transform<&mut Retain>`][4] |
/// | Insert | Delete | [`Compose<&mut Delete>`][5] | [`Transform<&mut Delete>`][6] |
///
/// [1]: ../trait.Compose.html#impl-Compose<%26mut+Insert<T,+A>>-for-U
/// [2]: #impl-Transform<%26mut+Insert<T,+A>>-for-%26mut+Insert<T,+A>
/// [3]: #impl-Compose<%26mut+Retain<A>>-for-%26mut+Insert<T,+A>
/// [4]: #impl-Transform<%26mut+Retain<A>>-for-%26mut+Insert<T,+A>
/// [5]: #impl-Compose<%26mut+Delete>-for-%26mut+Insert<T,+A>
/// [6]: #impl-Transform<%26mut+Delete>-for-%26mut+Insert<T,+A>
///
/// Apart from these traits, [`Insert<T, A>`] also implements [`Len`] and
/// [`Split`].
#[derive(Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Insert<T, A> {
    /// Contains the value that this operation inserts into a
    /// [`Delta`](super::Delta). Note that this doesn't necessarily need to be
    /// text. Any type that conforms to [`Value`](super::Value) (i.e. any
    /// countable type) is suitable as value.
    pub insert: T,

    /// Optionally contains the attributes of the elements in this insert
    /// sequence. If this field is `None`, it will eagerly assume other
    /// operations' attributes (regardless of this operation's priority) and
    /// therefore has different semantics than if this field were to be
    /// `Some(_)`, which always takes precedence if the given operation has
    /// priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<A>,
}

impl<T, A> Insert<T, A>
where
    T: Len,
{
    /// Returns a new retain operation with the same length as this insert's
    /// sequence's.
    pub fn as_retain(&self) -> Retain<A> {
        Retain {
            retain: self.len(),
            attributes: None,
        }
    }
}

impl<T, A> Len for Insert<T, A>
where
    T: Len,
{
    fn len(&self) -> usize {
        self.insert.len()
    }
}

impl<T, A> Split for Insert<T, A>
where
    T: Seq,
    A: Clone,
{
    fn split(&mut self, len: usize) -> Self {
        let remainder = self.insert.iter().take(len).collect();
        self.insert = self.insert.iter().skip(len).collect();

        Insert {
            insert: remainder,
            attributes: self.attributes.clone(),
        }
    }
}

/// Represents an operation that retains a sequence and optionally updates its
/// attributes.
///
/// For more information about the compositions and transformations of this
/// operation, read the documentation on the following trait implementations:
///
/// | Alice  | Bob    | Composition | Transformation |
/// |:-------|:-------|:------------|:---------------|
/// | Retain | Insert | [`Compose<&mut Insert>`][1] | [`Transform<&mut Insert>`][2] |
/// | Retain | Retain | [`Compose<&mut Retain>`][3] | [`Transform<&mut Retain>`][4] |
/// | Retain | Delete | [`Compose<&mut Delete>`][5] | [`Transform<&mut Delete>`][6] |
///
/// [1]: ../trait.Compose.html#impl-Compose<%26mut+Retain<A>>-for-U
/// [2]: #impl-Transform<%26mut+Insert<T,+A>>-for-%26mut+Retain<A>
/// [3]: #impl-Compose<%26mut+Retain<A>>-for-%26mut+Retain<A>
/// [4]: #impl-Transform<%26mut+Retain<A>>-for-%26mut+Retain<A>
/// [5]: #impl-Compose<%26mut+Delete>-for-%26mut+Retain<A>
/// [6]: #impl-Transform<%26mut+Delete>-for-%26mut+Retain<A>
///
/// Apart from these traits, [`Retain<T, A>`] also implements [`Len`] and
/// [`Split`].
#[derive(Arbitrary, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Retain<A> {
    /// Contains the number of elements to retain.
    pub retain: usize,

    /// Optionally contains the attributes that the elements in this sequence
    /// should be updated with. If this field is `None`, it will eagerly assume
    /// other operations' attributes (regardless of this operation's priority)
    /// and therefore has different semantics than if this field were to be
    /// `Some(_)`, which always takes precedence if the given operation has
    /// priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<A>,
}

impl<A> Retain<A> {
    /// Coalesces this retain's attributes with the given retain's attributes.
    pub fn or(self, other: Retain<A>) -> Retain<A> {
        Retain {
            retain: self.retain,
            attributes: self.attributes.or(other.attributes),
        }
    }
}

impl<A> Len for Retain<A> {
    fn len(&self) -> usize {
        self.retain
    }
}

impl<A> Split for Retain<A>
where
    A: Clone,
{
    fn split(&mut self, len: usize) -> Self {
        self.retain -= len;

        Retain {
            retain: len,
            attributes: self.attributes.clone(),
        }
    }
}

/// Represents an operation that deletes a sequence.
///
/// For more information about the compositions and transformations of this
/// operation, read the documentation on the following trait implementations:
///
/// | Alice  | Bob    | Composition | Transformation |
/// |:-------|:-------|:------------|:---------------|
/// | Delete | Insert | [`Compose<&mut Insert>`][1] | [`Transform<&mut Insert>`][2] |
/// | Delete | Retain | [`Compose<&mut Retain>`][3] | [`Transform<&mut Retain>`][4] |
/// | Delete | Delete | [`Compose<&mut Delete>`][5] | [`Transform<&mut Delete>`][6] |
///
/// [1]: ../trait.Compose.html#impl-Compose<%26mut+Delete>-for-U
/// [2]: #impl-Transform<%26mut+Insert<T,+A>>-for-%26mut+Delete
/// [3]: #impl-Compose<%26mut+Retain<A>>-for-%26mut+Delete
/// [4]: #impl-Transform<%26mut+Retain<A>>-for-%26mut+Delete
/// [5]: #impl-Compose<%26mut+Delete>-for-%26mut+Delete
/// [6]: #impl-Transform<%26mut+Delete>-for-%26mut+Delete
///
/// Apart from these traits, [`Delete<T, A>`] also implements [`Len`] and
/// [`Split`].
#[derive(Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delete {
    /// Contains the number of elements to delete.
    pub delete: usize,
}

impl Len for Delete {
    fn len(&self) -> usize {
        self.delete
    }
}

impl Split for Delete {
    fn split(&mut self, len: usize) -> Self {
        self.delete -= len;

        Delete { delete: len }
    }
}
