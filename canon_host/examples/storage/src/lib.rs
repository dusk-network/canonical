// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(lang_items)]

use canonical::{Canon, Store};
use canonical_derive::Canon;
use nstack::NStack;

#[derive(Canon, Debug, Clone)]
pub struct Storage<S: Store>(NStack<u8, (), S>);

impl<S: Store> Storage<S> {
    pub fn new() -> Self {
        Storage(NStack::new())
    }
}

extern "C" {
    fn sig(ofs: &u8, len: i32);
}

pub fn signal(msg: &str) {
    unsafe {
        let bytes = msg.as_bytes();
        sig(&bytes[0], bytes.len() as i32);
    }
}

trait ResultExt {
    type Output;
    fn unwrap_or_signal(self, msg: &str) -> Self::Output;
}

impl<T, E> ResultExt for Result<T, E> {
    type Output = T;

    fn unwrap_or_signal(self, msg: &str) -> T {
        match self {
            Ok(t) => t,
            Err(_) => {
                signal(msg);
                loop {}
            }
        }
    }
}

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Id32, Store};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

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
        let store = BS::default();
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
            _ => panic!("the tranasact"),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        let _ = transaction(bytes);
    }

    mod panic_handling {
        use super::signal;
        use core::panic::PanicInfo;

        #[panic_handler]
        fn panic(panic_info: &PanicInfo) -> ! {
            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                signal(s);
            }
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

        pub fn pop(
        ) -> Transaction<TransactionIndex, Result<Option<u8>, S::Error>>
        {
            Transaction::new(0xaab)
        }
    }
}
