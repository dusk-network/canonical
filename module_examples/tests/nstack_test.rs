// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical_host::{ExternalResolver, MemStore as MS, Wasm};

use microkelvin::Cardinality;
use nstack::NStack;
use nstack_module::Stack;

#[test]
fn push_pop() {
    let bytes = include_bytes!("../modules/nstack/nstack_module.wasm");

    let store = MS::new();

    let mut n_stack = NStack::<_, Cardinality, MS>::new();
    let mut wasm_stack = Wasm::new(Stack::new(), bytes);

    let n = 64;

    // push n numbers

    for i in 0..n {
        n_stack.push(i).unwrap();

        wasm_stack
            .transact(
                &Stack::<MS>::push(i),
                store.clone(),
                None::<ExternalResolver>,
            )
            .unwrap();
    }

    // pop n numbers

    for i in 0..n {
        let inv = n - i - 1;

        let popped = wasm_stack
            .transact(
                &Stack::<MS>::pop(),
                store.clone(),
                None::<ExternalResolver>,
            )
            .unwrap();

        assert_eq!(popped, Some(inv))
    }

    // assert empty

    assert_eq!(
        wasm_stack.transact(&Stack::<MS>::pop(), store).unwrap(),
        None
    );
}
