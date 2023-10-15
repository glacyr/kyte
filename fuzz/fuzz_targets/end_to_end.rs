#![no_main]

use kyte::{Compose, Delta, LastWriteWins, Transform};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (
    Delta::<String, LastWriteWins<usize>>,
    Delta::<String, LastWriteWins<usize>>,
    Delta::<String, LastWriteWins<usize>>,
)| {
    let before = data.0.into_iter().collect::<Delta<_, _>>();
    let alice = data.1.into_iter().collect::<Delta<_, _>>();
    let bob = data.2.into_iter().collect::<Delta<_, _>>();

    let alice_bob = before
        .clone()
        .compose(alice.clone())
        .compose(alice.clone().transform(bob.clone(), true));

    let bob_alice = before
        .clone()
        .compose(bob.clone())
        .compose(bob.clone().transform(alice.clone(), false));

    assert_eq!(alice_bob, bob_alice);
});
