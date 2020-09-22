// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

#![cfg_attr(feature = "hosted", no_std)]
#![feature(lang_items)]

use canonical::{Canon, Store};
use canonical_collections::Stack;
use canonical_derive::Canon;

#[derive(Canon, Debug)]
pub struct Storage<S: Store>(Stack<u8, S>);

impl<S: Store> Storage<S> {
    pub fn new() -> Self {
        Storage(Stack::new())
    }
}

#[cfg(feature = "hosted")]
impl<S: Store> Storage<S> {
    pub fn push(&mut self, value: u8) -> Result<(), S::Error> {
        self.0.push(value)
    }

    pub fn pop(&mut self) -> Result<Option<u8>, S::Error> {
        self.0.pop()
    }
}

#[cfg(feature = "hosted")]
mod bridge {
    use super::*;

    use canonical::{BridgeStore, ByteSink, Store};

    const PAGE_SIZE: usize = 1024 * 64;

    type BS = BridgeStore<[u8; 8]>;

    fn query(_bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        Ok(())
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        //
    }

    fn transaction(
        bytes: &mut [u8; PAGE_SIZE],
    ) -> Result<(), <BS as Store>::Error> {
        let source = &mut &bytes[..];

        let store = BS::singleton();

        // read self.
        let mut slf: Storage<BS> = Canon::<BS>::read(source)?;
        // read transaction id
        let tid: u16 = Canon::<BS>::read(source)?;

        match tid {
            // push
            0xaaa => {
                let t: i32 = Canon::<BS>::read(source)?;
                slf.0.push(t)?;
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // no return value
                Ok(())
            }
            // pop
            0xaab => {
                let t_opt = slf.pop();
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // write return value
                Canon::<BS>::write(&t_opt, &mut sink)
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }

    #[cfg(feature = "hosted")]
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
    use canonical_host::{Module, Transaction};

    impl<S: Store> Module for Storage<S> {
        const BYTECODE: &'static [u8] = include_bytes!("../storage.wasm");
    }

    // transactions
    type TransactionIndex = u16;

    impl<S: Store> Storage<S> {
        pub fn push(i: u8) -> Transaction<(TransactionIndex, u8), ()> {
            Transaction::new((0xaaa, i))
        }

        pub fn pop() -> Transaction<TransactionIndex, Option<u8>> {
            Transaction::new(0xaab)
        }
    }
}
