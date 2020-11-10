// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical_host::{MemStore as MS, Wasm};

use nstack_module::Stack;

#[test]
fn push_pop() {
    let store = MS::new();
    let mut wasm_stack = Wasm::new(
        Stack::new(),
        include_bytes!("../modules/nstack/nstack_module.wasm"),
    );

    let n = 4;

    // push n numbers

    for i in 0..n {
        wasm_stack
            .transact(&Stack::<MS>::push(i), store.clone())
            .unwrap();
    }

    // pop n numbers

    for i in 0..n {
        let inv = n - i - 1;
        assert_eq!(
            wasm_stack
                .transact(&Stack::<MS>::pop(), store.clone())
                .unwrap(),
            Some(inv)
        );
    }

    // assert empty

    assert_eq!(
        wasm_stack
            .transact(&Stack::<MS>::pop(), store.clone())
            .unwrap(),
        None
    );
}
