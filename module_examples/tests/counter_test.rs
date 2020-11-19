// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical_host::{MemStore, Remote, Wasm};

use counter::Counter;

#[test]
fn query() {
    let store = MemStore::new();
    let wasm_counter = Wasm::new(
        Counter::new(99),
        include_bytes!("../modules/counter/counter.wasm"),
    );

    let remote = Remote::new(wasm_counter, &store).unwrap();

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store.clone(), None)
            .unwrap(),
        99
    );

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::is_even(), store.clone(), None::<_>)
            .unwrap(),
        false
    );

    let (a, b) = (5, 2828);

    assert_eq!(
            remote
                .cast::<Wasm<Counter, MemStore>>()
                .unwrap()
    <<<<<<< HEAD
                .query(&Counter::xor_values(a, b), store)
    =======
                .query(&Counter::xor_values(a, b), store.clone(), None::<_>)
    >>>>>>> Add custom resolver / invoker
                .unwrap(),
            99 ^ a ^ b
        );
}

#[test]
fn transaction() {
    let store = MemStore::new();
    let wasm_counter = Wasm::new(
        Counter::new(99),
        include_bytes!("../modules/counter/counter.wasm"),
    );
    let mut remote = Remote::new(wasm_counter, &store).unwrap();

    // note, there's no reason to do compare and swap here,
    // just for testing return values from transactions

    let mut cast = remote.cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    assert!(cast
        .transact(&Counter::compare_and_swap(99, 32), store.clone(), None::<_>)
        .unwrap());

    cast.commit().unwrap();
    // assert cas was successful

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store.clone(), None)
            .unwrap(),
        32
    );

    let mut cast = remote.cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::increment(), store.clone(), None)
        .unwrap();
    cast.commit().unwrap();

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store.clone(), None)
            .unwrap(),
        33
    );

    let mut cast = remote.cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::decrement(), store.clone(), None)
        .unwrap();
    cast.commit().unwrap();

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store.clone(), None)
            .unwrap(),
        32
    );

    let mut cast = remote.cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::adjust(-10), store.clone(), None)
        .unwrap();
    cast.commit().unwrap();

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store)
            .unwrap(),
        22
    );
}
