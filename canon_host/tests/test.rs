// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use canonical::Canon;
use canonical_derive::Canon;
use canonical_host::{MemStore, Query, Remote, Transact};

#[derive(Clone, Canon)]
struct Switch(bool);

#[derive(Clone, Canon)]
struct ReadState;

#[derive(Clone, Canon)]
struct ToggleState;

#[derive(Clone, Canon)]
enum Adjust {
    Increment,
    Decrement,
}


impl Query for Switch {
    type Args = ReadState;
    type Return = bool;

    fn query(&self, _: &ReadState) -> bool {
        self.0
    }
}

impl Transact for Switch {
    type Args = ToggleState;
    type Return = ();

    fn transact(&mut self, _: &ToggleState) -> () {
        self.0 = !self.0
    }
}

#[derive(Clone, Canon)]
struct Counter(i128);

impl Query for Counter {
    type Args = ReadState;
    type Return = i128;

    fn query(&self, _: &ReadState) -> i128 {
        self.0
    }
}

impl Transact for Counter {
    type Args = Adjust;
    type Return = ();

    fn transact(&mut self, arg: &Adjust) -> () {
        match arg {
            Adjust::Increment => self.0 += 1,
            Adjust::Decrement => self.0 -= 1,
        }
    }
}

#[test]
fn switches() {
    let mut world = Vec::new();

    world.push(Switch(true));
    world.push(Switch(false));

    assert_eq!(world[0].query(&ReadState), true);
    assert_eq!(world[1].query(&ReadState), false);

    world[0].transact(&ToggleState);

    assert_eq!(world[0].query(&ReadState), false);
}

#[test]
fn remote() {
    let mut world = Vec::new();

    let store = MemStore::new();

    world.push(Remote::new(Switch(true), &store).unwrap());
    world.push(Remote::new(Switch(true), &store).unwrap());

    assert_eq!(world[0].cast::<Switch>().query(&ReadState).unwrap(), true);

    world[0].cast_mut::<Switch>().transact(&ToggleState).unwrap();

    assert_eq!(world[0].cast::<Switch>().query(&ReadState).unwrap(), false);
}


#[test]
fn mixed_contracts() {
    let mut world = Vec::new();
    let store = MemStore::new();

    world.push(Remote::new(Switch(true), &store).unwrap());
    world.push(Remote::new(Counter(32), &store).unwrap());

    // toggle first contract as above

    assert_eq!(world[0].cast::<Switch>().query(&ReadState).unwrap(), true);
    world[0].cast_mut::<Switch>().transact(&ToggleState).unwrap();
    assert_eq!(world[0].cast::<Switch>().query(&ReadState).unwrap(), false);

    // let's play with the counter

    assert_eq!(world[1].cast::<Counter>().query(&ReadState).unwrap(), 32);

    world[1].cast_mut::<Counter>().transact(&Adjust::Increment).unwrap();
    world[1].cast_mut::<Counter>().transact(&Adjust::Increment).unwrap();
    world[1].cast_mut::<Counter>().transact(&Adjust::Decrement).unwrap();

    assert_eq!(world[1].cast::<Counter>().query(&ReadState).unwrap(), 33);
}
