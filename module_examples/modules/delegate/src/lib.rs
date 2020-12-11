// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(lang_items)]
#![feature(min_const_generics)]

use canonical_derive::Canon;

// query ids
pub const DELEGATE_READ_VALUE: u8 = 0;

// transaction ids
pub const DELEGATE_ADJUST: u8 = 0;

#[derive(Clone, Canon, Debug)]
pub struct Delegator;

impl Delegator {
    pub fn new() -> Self {
        Delegator
    }
}

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Canon, Id32, Store};

    use counter::Counter;

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;
    type ContractAddr = u64;

    impl Delegator {
        pub fn delegate_read_value(&self, addr: ContractAddr) -> i32 {
            let query = Counter::read_value();
            todo!()
        }

        pub fn delegate_adjust(&mut self, addr: ContractAddr, by: i32) {
            todo!()
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let store = BS::default();
        let mut source = ByteSource::new(&bytes[..], &store);

        // read self.
        let slf: Delegator = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            DELEGATE_READ_VALUE => {
                let addr: ContractAddr = Canon::<BS>::read(&mut source)?;
                let ret = slf.delegate_read_value(addr);
                let mut sink = ByteSink::new(&mut bytes[..], &store);
                Canon::<BS>::write(&ret, &mut sink)?;
                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        let _ = query(bytes);
    }

    fn transaction(
        bytes: &mut [u8; PAGE_SIZE],
    ) -> Result<(), <BS as Store>::Error> {
        let store = BS::default();
        let mut source = ByteSource::new(bytes, &store);

        // read self.
        let mut slf: Delegator = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            // increment (&Self)
            DELEGATE_TRANSACTION => {
                let mut sink = ByteSink::new(&mut bytes[..], &store);
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // no return value
                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }

    include!("../../../../canon_module/src/panic_handling.rs");
}

#[cfg(feature = "host")]
mod host {
    use super::*;
    use canonical::{Query, Transaction};

    impl Delegator {
        pub fn delegate_read_value(
            addr: u64,
        ) -> Query<Self, u64, i32, DELEGATE_READ_VALUE> {
            Query::new(addr)
        }

        pub fn delegate_adjust(
            addr: u64,
            by: i32,
        ) -> Transaction<Self, (u64, i32), (), DELEGATE_ADJUST> {
            Transaction::new((addr, by))
        }
    }
}
