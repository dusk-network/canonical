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

// #[derive(Clone, Canon)]
// struct ReadState;

// #[derive(Clone, Canon)]
// struct ToggleState;

// #[derive(Clone, Canon)]
// enum Adjust {
//     Increment,
//     Decrement,
// }

// impl Query for Switch {
//     type Args = ReadState;
//     type Return = bool;

//     fn query(&self, _: &ReadState) -> bool {
//         self.0
//     }
// }

// impl Transact for Switch {
//     type Args = ToggleState;
//     type Return = ();

//     fn transact(&mut self, _: &ToggleState) -> () {
//         self.0 = !self.0
//     }
// }

#[derive(Clone, Canon)]
struct Counter(i128);

impl Counter {
    fn read_state(&self) -> i128 {
        self.0
    }

    fn adjust_state(&mut self, by: i128) {
        self.0 += by
    }
}

// impl Query for Counter {
//     type Args = ReadState;
//     type Return = i128;

//     fn query(&self, _: &ReadState) -> i128 {
//         self.0
//     }
// }

// impl Transact for Counter {
//     type Args = Adjust;
//     type Return = ();

//     fn transact(&mut self, arg: &Adjust) -> () {
//         match arg {
//             Adjust::Increment => self.0 += 1,
//             Adjust::Decrement => self.0 -= 1,
//         }
//     }
// }

#[test]
fn remote() {
    let mut world = Vec::new();

    let store = MemStore::new();

    world.push(Remote::new(Switch(true), &store).unwrap());
    world.push(Remote::new(Switch(true), &store).unwrap());

    assert_eq!(
        world[0].cast::<Switch>().as_ref().unwrap().read_state(),
        true
    );

    let mut transaction = world[0].cast_mut::<Switch>().unwrap();
    transaction.toggle();
    transaction.commit();

    assert_eq!(
        world[0].cast::<Switch>().as_ref().unwrap().read_state(),
        false
    );
}

// #[test]
// fn mixed_contracts() {
//     let mut world = Vec::new();
//     let store = MemStore::new();

//     world.push(Remote::new(Switch(true), &store).unwrap());
//     world.push(Remote::new(Counter(32), &store).unwrap());

//     assert_eq!(world[0].cast::<Switch>().unwrap().read_state(), true);
//     world[0].cast_mut::<Switch>().unwrap();
//     assert_eq!(world[0].cast::<Switch>().query(&ReadState).unwrap(), false);

//     // let's play with the counter

//     assert_eq!(world[1].cast::<Counter>().query(&ReadState).unwrap(), 32);

//     world[1].cast_mut::<Counter>().unwrap().adjust(1);
//     world[1].cast_mut::<Counter>().unwrap().adjust(1);
//     world[1].cast_mut::<Counter>().unwrap().adjust(-1);

//     assert_eq!(world[1].cast::<Counter>().unwrap().read_state(), 33);
// }
