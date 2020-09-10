// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

#![cfg_attr(feature = "wasm", no_std)]
#![feature(lang_items)]

use canonical::Canon;
use canonical_derive::Canon;

#[derive(Clone, Canon, Default)]
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

#[cfg(feature = "wasm")]
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

#[cfg(feature = "wasm")]
mod bridge {
    use super::*;

    use canonical::{BridgeStore, CanonError};

    const PAGE_SIZE: usize = 1024 * 64;

    type Store = BridgeStore<[u8; 8]>;

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let source = &mut &bytes[..];
        // read self.
        let slf: Counter = Canon::<Store>::read(source)?;
        // read query id
        let qid: u16 = Canon::<Store>::read(source)?;
        match qid {
            // read_value (&Self) -> i32
            0 => {
                let ret = slf.read_value();

                let sink = &mut &mut bytes[..];
                Canon::<Store>::write(&ret, sink)?;
                Ok(())
            }
            // xor_values (&Self, a: i32, b: i32) -> i32
            1 => {
                let (a, b): (i32, i32) = Canon::<Store>::read(source)?;
                let ret = slf.xor_values(a, b);

                let sink = &mut &mut bytes[..];
                Canon::<Store>::write(&ret, sink)?;
                Ok(())
            }
            // xor_value (&Self) -> bool
            2 => {
                let ret = slf.is_even();

                let sink = &mut &mut bytes[..];
                Canon::<Store>::write(&ret, sink)?;
                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        query(bytes).unwrap()
    }

    fn transaction(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let source = &mut &bytes[..];

        // read self.
        let mut slf: Counter = Canon::<Store>::read(source)?;
        // read transaction id
        let qid: u16 = Canon::<Store>::read(source)?;
        match qid {
            // increment (&Self)
            0 => {
                slf.increment();
                let sink = &mut &mut bytes[..];
                // return new state
                Canon::<Store>::write(&slf, sink)?;
                // no return value
                Ok(())
            }
            1 => {
                // no args
                slf.decrement();
                let sink = &mut &mut bytes[..];
                // return new state
                Canon::<Store>::write(&slf, sink)?;
                // no return value
                Ok(())
            }
            2 => {
                // read arg
                let by: i32 = Canon::<Store>::read(source)?;
                slf.adjust(by);
                let sink = &mut &mut bytes[..];
                // return new state
                Canon::<Store>::write(&slf, sink)?;
                // no return value
                Ok(())
            }
            3 => {
                // read multiple args
                let (a, b): (i32, i32) = Canon::<Store>::read(source)?;
                let res = slf.compare_and_swap(a, b);
                let sink = &mut &mut bytes[..];
                // return new state
                Canon::<Store>::write(&slf, sink)?;
                // return result
                Canon::<Store>::write(&res, sink)
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }

    #[cfg(feature = "wasm")]
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
    use canonical_host::{Module, Query, Transaction};

    impl Module for Counter {
        const BYTECODE: &'static [u8] = include_bytes!("../counter.wasm");
    }

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
