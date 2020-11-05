// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical_host::{MemStore as Mem, Remote, Wasm};
use storage::Storage;

#[test]
#[ignore]
fn storage() {
    let store = Mem::new();

    let wasm_counter = Wasm::new(
        Storage::<Mem>::new(),
        include_bytes!("../examples/storage/storage.wasm"),
    );

    let mut remote = Remote::new(wasm_counter, &store).unwrap();

    let n = 4;

    let mut cast = remote.cast_mut::<Wasm<Storage<Mem>, Mem>>().unwrap();

    // push n values
    for i in 0..n {
        let val = i + 0xb0;
        cast.transact(&Storage::<Mem>::push(val), store.clone())
            .unwrap()
    }

    // pop n values
    for i in 0..n {
        // reverse order
        let val = n - i - 1 + 0xb0;
        assert_eq!(
            cast.transact(&Storage::<Mem>::pop(), store.clone())
                .unwrap()
                .unwrap(),
            Some(val)
        )
    }

    // empty
    assert_eq!(
        cast.transact(&Storage::<Mem>::pop(), store.clone())
            .unwrap()
            .unwrap(),
        None
    );
}
