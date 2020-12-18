// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical_host::MemStore;
use canonical_module::{
    wasm, Apply, Execute, Module, Query, TestResolver, Transaction,
};
use counter::{self, Counter};

#[test]
fn query() {
    let store = MemStore::new();
    let wasm_counter = wasm::Wasm::new(
        Counter::new(99),
        store.clone(),
        include_bytes!("../modules/counter/counter.wasm"),
        TestResolver,
    );

    let remote = Module::new(wasm_counter, store.clone()).unwrap();

    let query: Query<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Query<Counter, (), i32, { counter::READ_VALUE }>,
        i32,
        { wasm::WASM_QUERY },
    > = Query::new(Counter::read_value());

    let cast: wasm::Wasm<Counter, TestResolver, MemStore> =
        remote.cast().unwrap();
    assert_eq!(cast.execute(query).unwrap(), 99);

    let query: Query<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Query<Counter, (), bool, { counter::IS_EVEN }>,
        bool,
        { wasm::WASM_QUERY },
    > = Query::new(Counter::is_even());
    let cast: wasm::Wasm<Counter, TestResolver, MemStore> =
        remote.cast().unwrap();
    assert_eq!(cast.execute(query).unwrap(), false);

    let (a, b) = (5, 2828);

    let query = Query::<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Query<Counter, (i32, i32), i32, { counter::XOR_VALUES }>,
        i32,
        { wasm::WASM_QUERY },
    >::new(Counter::xor_values(a, b));
    let cast: wasm::Wasm<Counter, TestResolver, MemStore> =
        remote.cast().unwrap();
    assert_eq!(cast.execute(query).unwrap(), 99 ^ a ^ b);
}

#[test]
fn transaction() {
    let store = MemStore::new();
    let wasm_counter = wasm::Wasm::new(
        Counter::new(99),
        store.clone(),
        include_bytes!("../modules/counter/counter.wasm"),
        TestResolver,
    );
    let mut remote = Module::new(wasm_counter, store.clone()).unwrap();

    // note, there's no reason to do compare and swap here,
    // just for testing return values from transactions

    let transaction: Transaction<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Transaction<Counter, (i32, i32), bool, { counter::COMPARE_AND_SWAP }>,
        bool,
        { wasm::WASM_TRANSACTION },
    > = Transaction::new(Counter::compare_and_swap(99, 32));

    let mut cast = remote
        .cast_mut::<wasm::Wasm<Counter, TestResolver, MemStore>>()
        .unwrap();
    assert_eq!(cast.apply(transaction).unwrap(), true);

    // assert cas was successful

    let query: Query<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Query<Counter, (), i32, { counter::READ_VALUE }>,
        i32,
        { wasm::WASM_QUERY },
    > = Query::new(Counter::read_value());

    let cast: wasm::Wasm<Counter, TestResolver, MemStore> =
        remote.cast().unwrap();
    assert_eq!(cast.execute(query).unwrap(), 32);

    // increment

    let transaction: Transaction<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Transaction<Counter, (), (), { counter::INCREMENT }>,
        (),
        { wasm::WASM_TRANSACTION },
    > = Transaction::new(Counter::increment());
    let mut cast = remote
        .cast_mut::<wasm::Wasm<Counter, TestResolver, MemStore>>()
        .unwrap();
    cast.apply(transaction).unwrap();

    // read incremented

    let query: Query<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Query<Counter, (), i32, { counter::READ_VALUE }>,
        i32,
        { wasm::WASM_QUERY },
    > = Query::new(Counter::read_value());

    let cast: wasm::Wasm<Counter, TestResolver, MemStore> =
        remote.cast().unwrap();
    assert_eq!(cast.execute(query).unwrap(), 33);

    // decrement

    let transaction: Transaction<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Transaction<Counter, (), (), { counter::DECREMENT }>,
        (),
        { wasm::WASM_TRANSACTION },
    > = Transaction::new(Counter::decrement());
    let mut cast = remote
        .cast_mut::<wasm::Wasm<Counter, TestResolver, MemStore>>()
        .unwrap();
    cast.apply(transaction).unwrap();

    // check that vaule decremented

    let query: Query<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Query<Counter, (), i32, { counter::READ_VALUE }>,
        i32,
        { wasm::WASM_QUERY },
    > = Query::new(Counter::read_value());

    let cast: wasm::Wasm<Counter, TestResolver, MemStore> =
        remote.cast().unwrap();
    assert_eq!(cast.execute(query).unwrap(), 32);

    // adjust

    let transaction: Transaction<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Transaction<Counter, i32, (), { counter::ADJUST }>,
        (),
        { wasm::WASM_TRANSACTION },
    > = Transaction::new(Counter::adjust(-10));
    let mut cast = remote
        .cast_mut::<wasm::Wasm<Counter, TestResolver, MemStore>>()
        .unwrap();
    cast.apply(transaction).unwrap();

    // check adjusted

    let query: Query<
        wasm::Wasm<Counter, TestResolver, MemStore>,
        Query<Counter, (), i32, { counter::READ_VALUE }>,
        i32,
        { wasm::WASM_QUERY },
    > = Query::new(Counter::read_value());

    let cast: wasm::Wasm<Counter, TestResolver, MemStore> =
        remote.cast().unwrap();
    assert_eq!(cast.execute(query).unwrap(), 22);
}
