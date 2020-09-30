// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use canonical::{Handle, Store};
use canonical_host::MemStore;

#[test]
fn zero_sized_handles() {
    let store = MemStore::new();
    let handle = Handle::new(()).unwrap();

    let id = store.put(&handle).unwrap();

    let restored = store.get(&id).unwrap();
    assert_eq!((), restored);
}
