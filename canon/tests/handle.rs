// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

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
