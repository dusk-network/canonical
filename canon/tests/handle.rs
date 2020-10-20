// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Repr, Store};
use canonical_host::MemStore;

#[test]
fn zero_sized_reprs() {
    let store = MemStore::new();
    let repr = Repr::new(());

    let id = store.put(&repr).unwrap();

    let restored = store.get(&id).unwrap();
    assert_eq!((), restored);
}
