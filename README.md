# Kyte

Extensible mechanics for operational transformation in Rust that are generic
with respect to their value (not constrained to text), wire-compatible with
[Quill](https://quilljs.com/docs/delta/) and fully fuzzed.

Operational Transformation (OT) enables real-time collaborative editing by
enabling two (or more) users to make changes at the same time. An OT-capable
central server transforms and broadcasts these changes so everyone is
looking at the same synchronized state, even in the presence of severe
latency.

This library can be integrated to build both a client-side and/or
server-side implementation of operational transformation within your
application.

## Usage

```ignore
use kyte::{Compose, Delta, Transform};

let before = Delta::new().insert("Hello World".to_owned(), ());

let alice = Delta::new().retain(5, ()).insert(",".to_owned(), ());
let bob = Delta::new().retain(11, ()).insert("!".to_owned(), ());

assert_eq!(
    before
        .compose(alice)
        .compose(alice.transform(bob, true)),
    before
        .compose(bob)
        .compose(bob.transform(alice, false)),
)
```

## License

Copyright 2023 Glacyr B.V.

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
