// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(never_type)]

use canonical_host::{MemError, MemStore};
use canonical_module::{wasm, Execute, Module, Query, Signal, TestResolver};
use panic::{self, Panico};

#[test]
fn panic() {
    let store = MemStore::new();
    let wasm_panic = wasm::Wasm::new(
        Panico,
        store.clone(),
        include_bytes!("../modules/panic/panic.wasm"),
        TestResolver,
    );

    let remote = Module::new(wasm_panic, store.clone()).unwrap();
    let cast = remote
        .cast::<wasm::Wasm<Panico, TestResolver, MemStore>>()
        .unwrap();

    let query_a: Query<
        wasm::Wasm<Panico, TestResolver, MemStore>,
        Query<Panico, (), !, { panic::PANIC_A }>,
        !,
        { wasm::WASM_QUERY },
    > = Query::new(Panico::panic_a());

    match cast.execute(query_a) {
        Err(MemError::Signal(sig)) => {
            assert_eq!(
                sig,
                Signal::panic(
                    "panicked at \'let\'s panic!\', module_examples/modules/panic/src/lib.rs:31:13\n"
                )
            );
        }
        _ => panic!(),
    }

    let query_b: Query<
        wasm::Wasm<Panico, TestResolver, MemStore>,
        Query<Panico, (), !, { panic::PANIC_B }>,
        !,
        { wasm::WASM_QUERY },
    > = Query::new(Panico::panic_b());

    match cast.execute(query_b) {
        Err(MemError::Signal(sig)) => {
            assert_eq!(
                sig,
                Signal::panic(
                    "panicked at \'let\'s panic differently!\', module_examples/modules/panic/src/lib.rs:35:13\n"
                )
            );
        }
        _ => panic!(),
    }
}
