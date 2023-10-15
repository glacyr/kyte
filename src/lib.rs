#![warn(missing_docs)]
//! Extensible mechanics for operational transformation in Rust that are generic
//! with respect to their value (not constrained to text), wire-compatible with
//! [Quill](https://quilljs.com/docs/delta/) and fully fuzzed.
//!
//! Operational Transformation (OT) enables real-time collaborative editing by
//! enabling two (or more) users to make changes at the same time. An OT-capable
//! central server transforms and broadcasts these changes so everyone is
//! looking at the same synchronized state, even in the presence of severe
//! latency.
//!
//! This library can be integrated to build both a client-side and/or
//! server-side implementation of operational transformation within your
//! application.
//!
//! # Usage
//!
//! ```ignore
//! use kyte::{Compose, Delta, Transform};
//!
//! let before = Delta::new().insert("Hello World".to_owned(), ());
//!
//! let alice = Delta::new().retain(5, ()).insert(",".to_owned(), ());
//! let bob = Delta::new().retain(11, ()).insert("!".to_owned(), ());
//!
//! assert_eq!(
//!     before
//!         .compose(alice)
//!         .compose(alice.transform(bob, true)),
//!     before
//!         .compose(bob)
//!         .compose(bob.transform(alice, false)),
//! )
//! ```
//!
//! ## Acknowledgements
//!
//! This library largely implements Quill's delta
//! [spec](https://github.com/quilljs/delta/) and uses some of their
//! [test cases](https://github.com/quilljs/delta/tree/main/test/delta) for unit
//! testing. Simply put, this library wouldn't exist without their amazing work
//! on Quill.

mod compose;
mod delta;
mod iter;
mod op;
pub mod ops;
mod seq;
mod transform;

pub use compose::Compose;
#[doc(hidden)]
pub use compose::LastWriteWins;
pub use delta::Delta;
pub use iter::Iter;
pub use op::{Op, Split};
pub use seq::{Len, Seq};
pub use transform::Transform;

#[cfg(test)]
mod tests {
    use crate::LastWriteWins;

    use super::{Compose, Delta, Transform};

    #[test]
    fn test_end_to_end() {
        let before = Delta::new().insert("Hello World".to_owned(), ());

        let alice = Delta::new().retain(5, ()).insert(",".to_owned(), ());
        let bob = Delta::new().retain(11, ()).insert("!".to_owned(), ());

        assert_eq!(
            before
                .clone()
                .compose(alice.clone())
                .compose(alice.clone().transform(bob.clone(), true)),
            before
                .clone()
                .compose(bob.clone())
                .compose(bob.clone().transform(alice.clone(), false)),
        )
    }

    #[test]
    fn test_end_to_end_insert_retain_1() {
        let before = Delta::new().insert("0123456".to_owned(), ());

        let alice = Delta::new().retain(1000, ()).insert("6".to_owned(), ());
        let bob = Delta::new().insert("ABCD".to_owned(), ());

        assert_eq!(
            before
                .clone()
                .compose(alice.clone())
                .compose(alice.clone().transform(bob.clone(), true)),
            before
                .clone()
                .compose(bob.clone())
                .compose(bob.clone().transform(alice.clone(), false)),
        )
    }

    #[test]
    fn test_end_to_end_insert_retain_2() {
        let before = Delta::new().retain(5, ()).insert("ABCD".to_owned(), ());

        let alice = Delta::new().retain(5, ()).insert("ABCD".to_owned(), ());
        let bob = Delta::new().insert("ABC".to_owned(), ());

        assert_eq!(
            before
                .clone()
                .compose(alice.clone())
                .compose(alice.clone().transform(bob.clone(), true)),
            before
                .clone()
                .compose(bob.clone())
                .compose(bob.clone().transform(alice.clone(), false)),
        )
    }

    #[test]
    fn test_end_to_end_insert_retain_3() {
        let before = Delta::new()
            .retain(usize::MAX / 3, ())
            .retain(usize::MAX / 3, ())
            .retain(usize::MAX / 3, ())
            .retain(usize::MAX / 3, ())
            .retain(1429, ());

        let alice = Delta::new().insert("Hello, World!".to_owned(), ());
        let bob = Delta::new()
            .insert("Hello, World!".to_owned(), ())
            .delete(usize::MAX / 3);

        assert_eq!(
            before
                .clone()
                .compose(alice.clone())
                .compose(alice.clone().transform(bob.clone(), true)),
            before
                .clone()
                .compose(bob.clone())
                .compose(bob.clone().transform(alice.clone(), false)),
        )
    }

    #[test]
    fn test_end_to_end_insert_retain_4() {
        let before = Delta::new().insert("Hello, World!".to_owned(), LastWriteWins(42));

        let alice = Delta::new().retain(128, LastWriteWins(1));
        let bob = Delta::new();

        assert_eq!(
            before
                .clone()
                .compose(alice.clone())
                .compose(alice.clone().transform(bob.clone(), true)),
            before
                .clone()
                .compose(bob.clone())
                .compose(bob.clone().transform(alice.clone(), false)),
        )
    }

    #[test]
    fn test_end_to_end_insert_retain_5() {
        let before = Delta::<String, _>::new();

        let alice = Delta::new().retain(1, LastWriteWins(0usize));
        let bob = Delta::new().retain(1, LastWriteWins(42));

        assert_eq!(
            before
                .clone()
                .compose(alice.clone())
                .compose(alice.clone().transform(bob.clone(), true)),
            before
                .clone()
                .compose(bob.clone())
                .compose(bob.clone().transform(alice.clone(), false)),
        )
    }

    #[test]
    fn test_end_to_end_insert_retain_6() {
        let before = Delta::<String, _>::new();

        let alice = Delta::new().retain(4, None);
        let bob = Delta::new().retain(100, LastWriteWins(0));

        assert_eq!(
            before
                .clone()
                .compose(alice.clone())
                .compose(alice.clone().transform(bob.clone(), true)),
            before
                .clone()
                .compose(bob.clone())
                .compose(bob.clone().transform(alice.clone(), false)),
        )
    }
}
