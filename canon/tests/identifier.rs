// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, Id, Sink, Source};
use canonical_fuzz::fuzz_canon;

#[test]
fn test_empty() {
    let a = Id::raw(
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
        ],
        989,
    );
    let mut buf = vec![0u8; a.encoded_len()];
    let mut sink = Sink::new(&mut buf);
    a.encode(&mut sink);
    let mut source = Source::new(&buf);
    let b = Id::decode(&mut source).unwrap();
    assert_eq!(a, b);
}

#[test]
fn fuzz_id() {
    fuzz_canon::<Id>()
}
