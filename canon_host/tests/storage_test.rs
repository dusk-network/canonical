// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use canonical_host::{MemStore as Mem, Remote, Wasm};
use storage::Storage;

#[test]
fn storage() {
    let store = Mem::new();

    let wasm_counter = Wasm::new(Storage::new());

    let mut remote = Remote::new(wasm_counter, &store).unwrap();

    let n = 16;

    let mut cast = remote.cast_mut::<Wasm<Storage<Mem>, Mem>>().unwrap();

    // push n values
    for i in 0..n {
        let val = i + 0xb0;
        cast.transact(&Storage::<Mem>::push(val), store.clone())
            .unwrap()
            .unwrap()
            .unwrap()
    }

    // pop n values
    for i in 0..n {
        // reverse order
        let val = n - i - 1 + 0xb0;
        assert_eq!(
            cast.transact(&Storage::<Mem>::pop(), store.clone())
                .unwrap()
                .unwrap()
                .unwrap(),
            Some(val)
        )
    }

    // empty
    assert_eq!(
        cast.transact(&Storage::<Mem>::pop(), store.clone())
            .unwrap()
            .unwrap()
            .unwrap(),
        None
    );
}
