// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use canonical_host::{MemStore, Remote, Wasm};

use counter::Counter;

#[test]
fn query() {
    let mut world = Vec::new();

    let store = MemStore::new();

    let wasm_counter = Wasm::new(Counter::new(99));

    world.push(Remote::new(wasm_counter, &store).unwrap());

    assert_eq!(
        world[0]
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value())
            .unwrap()
            .unwrap(),
        99
    );

    assert_eq!(
        world[0]
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::is_even())
            .unwrap()
            .unwrap(),
        false
    );

    let (a, b) = (5, 2828);

    assert_eq!(
        world[0]
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::xor_values(a, b))
            .unwrap()
            .unwrap(),
        99 ^ a ^ b
    );
}

#[test]
fn transaction() {
    let mut world = Vec::new();

    let store = MemStore::new();

    let wasm_counter = Wasm::new(Counter::new(99));

    world.push(Remote::new(wasm_counter, &store).unwrap());

    // note, there's no reason to do compare and swap here, just for testing return values from transactions

    let mut cast = world[0].cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    assert!(cast
        .transact(&Counter::compare_and_swap(99, 32))
        .unwrap()
        .unwrap());

    cast.commit().unwrap();
    // assert cas was successful

    assert_eq!(
        world[0]
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value())
            .unwrap()
            .unwrap(),
        32
    );

    let mut cast = world[0].cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::increment()).unwrap().unwrap();
    cast.commit().unwrap();

    assert_eq!(
        world[0]
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value())
            .unwrap()
            .unwrap(),
        33
    );

    let mut cast = world[0].cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::decrement()).unwrap().unwrap();
    cast.commit().unwrap();

    assert_eq!(
        world[0]
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value())
            .unwrap()
            .unwrap(),
        32
    );

    let mut cast = world[0].cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::adjust(-10)).unwrap().unwrap();
    cast.commit().unwrap();

    assert_eq!(
        world[0]
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value())
            .unwrap()
            .unwrap(),
        22
    );
}
