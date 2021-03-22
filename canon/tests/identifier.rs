// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, Id, Source};
use canonical_fuzz::fuzz_canon;

#[test]
fn test_empty() {
    let mut source = Source::new(&[0, 0, 0]);
    let id = Id::decode(&mut source).unwrap();
    assert_eq!(id, Id::default());
}

#[test]
fn fuzz_id() {
    fuzz_canon::<Id>()
}
