// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::Canon;
use canonical_derive::Canon;
use canonical_module::module;

#[derive(Clone, Canon, Debug)]
pub struct Counter {
    junk: u32,
    value: i32,
}

module! {
    impl Counter {
        pub fn static_test_a() -> i32 {
            22
        }

        // pub fn static_test_b() -> i32 {
        //     21
        // }

        // pub fn new(value: i32) -> Self {
        //     Counter {
        //         value,
        //         junk: 0xffffffff,
        //     }
        // }

        // pub fn read_value(&self) -> i32 {
        //     self.value
        // }

        // pub fn xor_values(&self, a: i32, b: i32) -> i32 {
        //     self.value ^ a ^ b
        // }

        // pub fn is_even(&self) -> bool {
        //     self.value % 2 == 0
        // }

        // pub fn increment(&mut self) -> () {
        //     self.value += 1;
        // }

        // pub fn decrement(&mut self) -> () {
        //     self.value -= 1;
        // }

        // pub fn adjust(&mut self, by: i32) -> () {
        //     self.value += by;
        // }

        // pub fn compare_and_swap(&mut self, expected: i32, new: i32) -> bool {
        //     if self.value == expected {
        //         self.value = new;
        //         true
        //     } else {
        //         false
        //     }
        // }
    }
}

#[test]
fn hello() {
    assert_eq!(Counter::static_test_a(), 22);
    // assert_eq!(Counter::static_test_b(), 21);
}

// #[test]
// fn new() {
//     let mut count = Counter::new(32);

//     assert_eq!(count.read_value(), 32);

//     // count.adjust(-12);

//     // assert_eq!(count.read_value(), 20);
// }
