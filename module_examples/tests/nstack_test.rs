// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical_host::{wasm, Apply, MemStore as MS, Transaction, Wasm};

use nstack_module::{self, Stack};

#[test]
fn push_pop() {
    let bytes = include_bytes!("../modules/nstack/nstack_module.wasm");

    let store = MS::new();

    let mut wasm_stack = Wasm::new(Stack::new(), store.clone(), bytes);

    let n = 16;

    // push n numbers

    for i in 0..n {
        let transaction = Transaction::<
            wasm::Wasm<Stack<MS>, MS>,
            Transaction<Stack<MS>, i32, (), { nstack_module::PUSH }>,
            (),
            { wasm::WASM_TRANSACTION },
        >::new(Stack::push(i));

        wasm_stack.apply(transaction).unwrap()
    }

    // pop n numbers

    for i in 0..n {
        let inv = n - i - 1;

        let transaction = Transaction::<
            wasm::Wasm<Stack<MS>, MS>,
            Transaction<Stack<MS>, (), Option<i32>, { nstack_module::POP }>,
            Option<i32>,
            { wasm::WASM_TRANSACTION },
        >::new(Stack::pop());

        let popped = wasm_stack.apply(transaction).unwrap();

        assert_eq!(popped, Some(inv))
    }

    // assert empty

    let transaction = Transaction::<
        wasm::Wasm<Stack<MS>, MS>,
        Transaction<Stack<MS>, (), Option<i32>, { nstack_module::POP }>,
        Option<i32>,
        { wasm::WASM_TRANSACTION },
    >::new(Stack::pop());

    assert!(wasm_stack.apply(transaction).unwrap().is_none())
}
