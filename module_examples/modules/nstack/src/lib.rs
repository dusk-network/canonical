// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(lang_items)]

use canonical::{Canon, Store};
use canonical_derive::Canon;
use microkelvin::Cardinality;
use nstack::NStack;

#[derive(Clone, Canon, Debug)]
pub struct Stack<S: Store> {
    inner: NStack<i32, Cardinality, S>,
}

impl<S: Store + Store> Stack<S> {
    pub fn new() -> Self {
        Stack {
            inner: NStack::new(),
        }
    }
}

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Id32, Store};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl Stack<BS> {
        pub fn push(&mut self, t: i32) -> Result<(), <BS as Store>::Error> {
            self.inner.push(t)
        }

        pub fn pop(&mut self) -> Result<Option<i32>, <BS as Store>::Error> {
            self.inner.pop()
        }
    }

    fn transaction(
        bytes: &mut [u8; PAGE_SIZE],
    ) -> Result<(), <BS as Store>::Error> {
        let store = BS::default();
        let mut source = ByteSource::new(&bytes[..], store.clone());

        // read self.
        let mut slf: Stack<BS> = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u16 = Canon::<BS>::read(&mut source)?;
        match qid {
            // push (&mut self, t: i32) -> ()
            0 => {
                let to_push = Canon::<BS>::read(&mut source)?;
                slf.push(to_push)?;
                Ok(())
            }
            // pop (&mut self) -> Option<i32>
            1 => {
                let ret = slf.pop()?;
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // new self
                Canon::<BS>::write(&slf, &mut sink)?;
                // result
                Canon::<BS>::write(&ret, &mut sink)?;
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

    // mod panic_handling {
    //     use core::panic::PanicInfo;

    //     #[panic_handler]
    //     fn panic(_: &PanicInfo) -> ! {
    //         loop {}
    //     }

    //     #[lang = "eh_personality"]
    //     extern "C" fn eh_personality() {}
    // }
}

#[cfg(feature = "host")]
mod host {
    use super::*;
    use canonical::Store;
    use canonical_host::Transaction;

    // transactions
    type TransactionIndex = u16;

    impl<S: Store> Stack<S> {
        pub fn push(t: i32) -> Transaction<(TransactionIndex, i32), ()> {
            Transaction::new((0, t))
        }

        pub fn pop() -> Transaction<TransactionIndex, Option<i32>> {
            Transaction::new(1)
        }
    }
}
