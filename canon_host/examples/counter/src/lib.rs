// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

#![no_std]
use canonical::{Canon, CanonError, BridgeStore, Store};
use canonical_derive::Canon;

#[derive(Clone, Canon)]
pub struct Counter {
    value: i32,
}

#[gen_remote]
impl Counter {
    pub fn read(&self) -> i32 {
        self.value
    }

    pub fn adjust(&mut self, add: i32) {
        self.value += add;
    }
}



type Id = [u8; 32];
type St = BridgeStore<Id>;

#[no_mangle]
fn call(mut state: &[u8], args: &[u8]) -> Result<Id, CanonError<<St as Store>::Error>> {
    let mut this: Counter = Canon::<St>::read(&mut state)?;
    this._adjust(1);

    let store = BridgeStore::<Id>::new();
    let ident = store.stash(this);

    Ok(ident)
}
