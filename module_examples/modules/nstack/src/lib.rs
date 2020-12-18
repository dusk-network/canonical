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

// transaction ids
pub const PUSH: u8 = 0;
pub const POP: u8 = 1;

#[derive(Clone, Canon)]
pub struct Stack<S: Store> {
    inner: NStack<i32, Cardinality, S>,
}

impl<S: Store> Stack<S>
where
    S: Store,
{
    pub fn new() -> Self {
        Stack {
            inner: NStack::new(),
        }
    }
}

impl<S: Store> Default for Stack<S>
where
    S: Store,
{
    fn default() -> Self {
        Self::new()
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

        let mut source = ByteSource::new(&bytes[..], &store);

        // read self.
        let mut slf: Stack<BS> = Canon::<BS>::read(&mut source)?;

        // read transacion id
        let transaction_id: u8 = Canon::<BS>::read(&mut source)?;
        match transaction_id {
            // push (&mut self, t: i32) -> ()
            PUSH => {
                let to_push = Canon::<BS>::read(&mut source)?;
                slf.push(to_push)?;
                let mut sink = ByteSink::new(&mut bytes[..], &store);
                // new self
                Canon::<BS>::write(&slf, &mut sink)?;
                // result is ()
                Ok(())
            }
            // pop (&mut self) -> Option<i32>
            POP => {
                let ret = slf.pop()?;
                let mut sink = ByteSink::new(&mut bytes[..], &store);
                // new self
                Canon::<BS>::write(&slf, &mut sink)?;
                // result
                Canon::<BS>::write(&ret, &mut sink)?;

                Ok(())
            }
            _ => panic!("invalid transaction id {}", transaction_id),
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
    use canonical::Store;
    use canonical_module::Transaction;

    impl<S: Store> Stack<S> {
        pub fn push(t: i32) -> Transaction<Self, i32, (), PUSH> {
            Transaction::new(t)
        }

        pub fn pop() -> Transaction<Self, (), Option<i32>, POP> {
            Transaction::new(())
        }
    }
}
