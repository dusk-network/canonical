// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(never_type)]

use canonical_derive::Canon;
use canonical_module::module;

#[derive(Clone, Canon, Debug)]
pub struct Panico;

module! {
    impl Panico {
        pub fn panic_a(&self) -> ! {
            panic!("let's panic!");
        }

        pub fn panic_b(&self) -> ! {
            panic!("let's panic differently!");
        }
    }
}
