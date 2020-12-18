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
    use canonical_module::{Query, RawQuery};

    // FIXME use real counter use counter::Counter;
    #[derive(Canon, Clone)]
    struct CounterErsatz;
    const READ_VALUE: u8 = 0;

    impl CounterErsatz {
        pub fn read_value() -> Query<Self, (), i32, READ_VALUE> {
            Query::new(())
        }
    }

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;
    type ContractAddr = [u8; 8];

    extern "C" {
        fn c_query(buffer: &mut u8, addr_len: u8);
    }

    fn contract_query<R>(addr: ContractAddr, query: RawQuery) -> R
    where
        R: Canon<BS>,
    {
        let bs = BS::default();
        let mut buffer = [0u8; 128];
        let addr_len = addr.as_ref().len();
        buffer[0..addr_len].copy_from_slice(addr.as_ref());
        let mut sink = ByteSink::new(&mut buffer[addr_len..], &bs);
        Canon::<BS>::write(&query, &mut sink).unwrap();
        unsafe {
            c_query(&mut buffer[0], addr_len as u8);
        }
        let mut source = ByteSource::new(&buffer, &bs);
        Canon::<BS>::read(&mut source).unwrap()
    }

    impl Delegator {
        pub fn delegate_query(
            &self,
            addr: ContractAddr,
            query: RawQuery,
        ) -> i32 {
            contract_query(addr, query)
        }

        pub fn delegate_adjust(&mut self, _addr: ContractAddr, _by: i32) {
            // let query = Counter::adjust(by);
            // contract_transaction(addr, query)
            todo!("deleg")
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
            DELEGATE_QUERY => {
                let (addr, query): (ContractAddr, RawQuery) =
                    Canon::<BS>::read(&mut source)?;
                let ret = slf.delegate_query(addr, query);
                let mut sink = ByteSink::new(&mut bytes[..], &store);
                Canon::<BS>::write(&ret, &mut sink)?;
                Ok(())
            }
            _ => panic!("wonka"),
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

        // read self (no-op in this case).
        let mut slf: Delegator = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let tid: u8 = Canon::<BS>::read(&mut source)?;
        match tid {
            DELEGATE_ADJUST => {
                let (addr, by): (ContractAddr, i32) =
                    Canon::<BS>::read(&mut source)?;
                let ret = slf.delegate_adjust(addr, by);
                let mut sink = ByteSink::new(&mut bytes[..], &store);
                // return new state (also no-op)
                Canon::<BS>::write(&slf, &mut sink)?;
                // return value (no-op)
                Canon::<BS>::write(&ret, &mut sink)
            }
            _ => panic!("wonk"),
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
    use canonical_module::{Query, Transaction};

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
