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
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Store};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<[u8; 8]>;

    impl Storage<BS> {
        pub fn push(&mut self, value: u8) -> Result<(), <BS as Store>::Error> {
            self.0.push(value)
        }

        pub fn pop(&mut self) -> Result<Option<u8>, <BS as Store>::Error> {
            self.0.pop()
        }
    }

    fn transaction(
        bytes: &mut [u8; PAGE_SIZE],
    ) -> Result<(), <BS as Store>::Error> {
        let store = BS::singleton();
        let mut source = ByteSource::new(&bytes[..], store.clone());

        // read self.
        let mut slf: Storage<BS> = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let tid: u16 = Canon::<BS>::read(&mut source)?;

        match tid {
            // push
            0xaaa => {
                let t: u8 = Canon::<BS>::read(&mut source)?;

                let res = slf.push(t);
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // write return value
                Canon::<BS>::write(&res, &mut sink)?;
                Ok(())
            }
            // pop
            0xaab => {
                // no arg to read
                let res = slf.pop();
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // write return value
                Canon::<BS>::write(&res, &mut sink)?;
                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        let _ = transaction(bytes);
    }

    mod panic_handling {
        use core::panic::PanicInfo;

        #[panic_handler]
        fn panic(_: &PanicInfo) -> ! {
            // overflow stack
            panic!()
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
        pub fn push(
            i: u8,
        ) -> Transaction<(TransactionIndex, u8), Result<(), S::Error>> {
            Transaction::new((0xaaa, i))
        }

        pub fn pop(
        ) -> Transaction<TransactionIndex, Result<Option<u8>, S::Error>>
        {
            Transaction::new(0xaab)
        }
    }
}
