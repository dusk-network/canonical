// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod common;
use common::no_externals;

use canonical_host::{MemError, MemStore, Remote, Signal, Wasm};
use panic::Panico;

#[test]
fn panic() {
    let host_externals = no_externals();

    let store = MemStore::new();
    let wasm_counter =
        Wasm::new(Panico, include_bytes!("../modules/panic/panic.wasm"));

    let remote = Remote::new(wasm_counter, &store).unwrap();
    let cast = remote.cast::<Wasm<Panico, MemStore>>().unwrap();

    match cast.query(&Panico::panic_a(), store.clone(), host_externals) {
        Err(MemError::Signal(sig)) => {
            assert_eq!(sig, Signal::panic("panicked at \'let\'s panic!\', module_examples/modules/panic/src/lib.rs:27:13\n"));
        }
        _ => panic!(),
    }

    match cast.query(&Panico::panic_b(), store, host_externals) {
        Err(MemError::Signal(sig)) => {
            assert_eq!(sig, Signal::panic("panicked at \'let\'s panic differently!\', module_examples/modules/panic/src/lib.rs:31:13\n"));
        }
        _ => panic!(),
    }
}
