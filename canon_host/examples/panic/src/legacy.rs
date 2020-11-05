// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(never_type)]

use canonical_derive::Canon;

#[derive(Clone, Canon, Debug)]
pub struct Panico;

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSource, Canon, Id32, Store};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl Panico {
        pub fn panic_a(&self) {
            panic!("let's panic!");
        }

        pub fn panic_b(&self) {
            panic!("let's panic differently!");
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let store = BS::default();
        let mut source = ByteSource::new(&bytes[..], store.clone());

        // no-op reading a unit struct;
        let slf = Panico::read(&mut source)?;

        // read msg len
        let tag = u16::read(&mut source)?;
        match tag {
            0 => slf.panic_a(),
            _ => slf.panic_b(),
        }
        Ok(())
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        query(bytes).unwrap();
    }
}

#[cfg(feature = "host")]
mod host {
    use super::*;
    use canonical_host::Query;

    // queries
    type QueryIndex = u16;

    impl Panico {
        pub fn panic_a() -> Query<QueryIndex, !> {
            Query::new(0)
        }

        pub fn panic_b() -> Query<QueryIndex, !> {
            Query::new(1)
        }
    }
}
