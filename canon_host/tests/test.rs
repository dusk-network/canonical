// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use canonical::Canon;
use canonical_derive::Canon;
use canonical_host::{MemStore, Remote};

#[derive(Clone, Canon)]
struct Switch(bool);

impl Switch {
    fn read_state(&self) -> bool {
        self.0
    }

    fn toggle(&mut self) {
        self.0 = !self.0
    }
}

#[derive(Clone, Canon)]
pub struct Counter(i128);

impl Counter {
    pub fn read_state(&self) -> i128 {
        self.0
    }

    pub fn adjust_state(&mut self, by: i128) {
        self.0 += by
    }
}

#[test]
fn remote() {
    let mut world = Vec::new();

    let store = MemStore::new();

    world.push(Remote::new(Switch(true), &store).unwrap());
    world.push(Remote::new(Switch(true), &store).unwrap());

    assert_eq!(world[0].query::<Switch>().unwrap().read_state(), true);

    let mut transaction = world[0].transact::<Switch>().unwrap();
    transaction.toggle();
    transaction.commit().unwrap();

    assert_eq!(world[0].query::<Switch>().unwrap().read_state(), false);
}
