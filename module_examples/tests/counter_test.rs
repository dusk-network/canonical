// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod common;
use common::no_externals;

use canonical_host::{MemStore, Remote, Wasm};
use counter::Counter;

#[test]
fn query() {
    let host_externals = no_externals();

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
            .query(&Counter::read_value(), store.clone(), host_externals)
            .unwrap(),
        99
    );

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::is_even(), store.clone(), host_externals)
            .unwrap(),
        false
    );

    let (a, b) = (5, 2828);

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::xor_values(a, b), store, host_externals)
            .unwrap(),
        99 ^ a ^ b
    );
}

#[test]
fn transaction() {
    let host_externals = no_externals();

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
        .transact(
            &Counter::compare_and_swap(99, 32),
            store.clone(),
            host_externals
        )
        .unwrap());

    cast.commit().unwrap();
    // assert cas was successful

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store.clone(), host_externals)
            .unwrap(),
        32
    );

    let mut cast = remote.cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::increment(), store.clone(), host_externals)
        .unwrap();
    cast.commit().unwrap();

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store.clone(), host_externals)
            .unwrap(),
        33
    );

    let mut cast = remote.cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::decrement(), store.clone(), host_externals)
        .unwrap();
    cast.commit().unwrap();

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store.clone(), host_externals)
            .unwrap(),
        32
    );

    let mut cast = remote.cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::adjust(-10), store.clone(), host_externals)
        .unwrap();
    cast.commit().unwrap();

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store, host_externals)
            .unwrap(),
        22
    );
}
