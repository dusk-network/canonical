// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Id32, Store};
use canonical_collections::Stack;
use canonical_host::MemStore;

#[test]
fn identifier_u64() {
    let a: u64 = 328;

    let id_a = MemStore::ident(&a);

    let store = MemStore::new();

    let id_b = store.put(&a).unwrap();

    assert!(id_a == id_b);
}

#[test]
fn identifier_stack() {
    let mut stack = Stack::new();

    let n = 8;

    for i in 0u64..n {
        stack.push(i).unwrap();
    }

    let id_a: Id32 = MemStore::ident(&stack);

    let store = MemStore::new();

    let id_b = store.put(&stack).unwrap();

    assert_eq!(id_a, id_b);
}
