// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical_host::{MemStore as MS, Remote, Wasm};

use nstack_module::Stack;

#[test]
fn push_pop() {
    let store = MS::new();
    let wasm_module = Wasm::new(
        Stack::new(),
        include_bytes!("../modules/nstack/nstack_module.wasm"),
    );
    let mut remote = Remote::new(wasm_module, &store).unwrap();

    let n = 1;

    // push n numbers

    for i in 0..n {
        let mut cast = remote.cast_mut::<Wasm<Stack<MS>, MS>>().unwrap();
        cast.transact(&Stack::<MS>::push(i), store.clone()).unwrap();
        cast.commit().unwrap();
    }

    // // pop n numbers

    // for i in 0..n {
    //     let inv = n - i - 1;

    //     let mut cast = remote.cast_mut::<Wasm<Stack, MS>>().unwrap();
    //     assert_eq!(
    //         cast.transact(&Stack::pop(), store.clone()).unwrap(),
    //         Some(inv)
    //     );
    //     cast.commit().unwrap();
    // }

    // assert empty

    let mut cast = remote.cast_mut::<Wasm<Stack<MS>, MS>>().unwrap();

    assert_eq!(
        cast.transact(&Stack::<MS>::pop(), store.clone()).unwrap(),
        None
    );
}
