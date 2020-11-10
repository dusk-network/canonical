// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(lang_items)]

use canonical::Canon;
use canonical_derive::Canon;

#[derive(Clone, Canon, Debug)]
pub struct Counter {
    junk: u32,
    value: i32,
}

impl Counter {
    pub fn new(value: i32) -> Self {
        Counter {
            value,
            junk: 0xffffffff,
        }
    }
}

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Id32, Store};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl Counter {
        pub fn read_value(&self) -> i32 {
            self.value
        }

        pub fn xor_values(&self, a: i32, b: i32) -> i32 {
            self.value ^ a ^ b
        }

        pub fn is_even(&self) -> bool {
            self.value % 2 == 0
        }

        pub fn increment(&mut self) {
            self.value += 1;
        }

        pub fn decrement(&mut self) {
            self.value -= 1;
        }

        pub fn adjust(&mut self, by: i32) {
            self.value += by;
        }

        pub fn compare_and_swap(&mut self, expected: i32, new: i32) -> bool {
            if self.value == expected {
                self.value = new;
                true
            } else {
                false
            }
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let store = BS::default();
        let mut source = ByteSource::new(&bytes[..], store.clone());

        // read self.
        let slf: Counter = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u16 = Canon::<BS>::read(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            0 => {
                let ret = slf.read_value();
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                Canon::<BS>::write(&ret, &mut sink)?;
                Ok(())
            }
            // xor_values (&Self, a: i32, b: i32) -> i32
            1 => {
                let (a, b): (i32, i32) = Canon::<BS>::read(&mut source)?;
                let ret = slf.xor_values(a, b);
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                Canon::<BS>::write(&ret, &mut sink)?;
                Ok(())
            }
            // xor_value (&Self) -> bool
            2 => {
                let ret = slf.is_even();
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());

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
        let mut source = ByteSource::new(bytes, store.clone());

        // read self.
        let mut slf: Counter = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let qid: u16 = Canon::<BS>::read(&mut source)?;
        match qid {
            // increment (&Self)
            0 => {
                slf.increment();
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // no return value
                Ok(())
            }
            1 => {
                // no args
                slf.decrement();
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // no return value
                Ok(())
            }
            2 => {
                // read arg
                let by: i32 = Canon::<BS>::read(&mut source)?;
                slf.adjust(by);
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // no return value
                Ok(())
            }
            3 => {
                // read multiple args
                let (a, b): (i32, i32) = Canon::<BS>::read(&mut source)?;
                let res = slf.compare_and_swap(a, b);
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // return result
                Canon::<BS>::write(&res, &mut sink)
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }

    mod panic_handling {
        use core::panic::PanicInfo;

        #[panic_handler]
        fn panic(_: &PanicInfo) -> ! {
            loop {}
        }

        #[lang = "eh_personality"]
        extern "C" fn eh_personality() {}
    }
}

#[cfg(feature = "host")]
mod host {
    use super::*;
    use canonical_host::{Query, Transaction};

    // queries
    type QueryIndex = u16;

    impl Counter {
        pub fn read_value() -> Query<QueryIndex, i32> {
            Query::new(0)
        }

        pub fn xor_values(
            a: i32,
            b: i32,
        ) -> Query<(QueryIndex, i32, i32), i32> {
            Query::new((1, a, b))
        }

        pub fn is_even() -> Query<QueryIndex, bool> {
            Query::new(2)
        }
    }

    // transactions
    type TransactionIndex = u16;

    impl Counter {
        pub fn increment() -> Transaction<TransactionIndex, ()> {
            Transaction::new(0)
        }

        pub fn decrement() -> Transaction<TransactionIndex, ()> {
            Transaction::new(1)
        }

        pub fn adjust(by: i32) -> Transaction<(TransactionIndex, i32), ()> {
            Transaction::new((2, by))
        }

        pub fn compare_and_swap(
            current: i32,
            new: i32,
        ) -> Transaction<(TransactionIndex, i32, i32), bool> {
            Transaction::new((3, current, new))
        }
    }
}
