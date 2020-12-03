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
    extern "C" {
        fn host_function(n: i32) -> i32;
    }

    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Id32, Store};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl Counter {
        pub fn read_value(&self) -> i32 {
            self.value
        }

        pub fn adjust(&mut self, by: i32) {
            self.value += unsafe { host_function(by) };
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
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }

    mod panic_handling {
        pub fn signal(msg: &str) {
            let bytes = msg.as_bytes();
            let len = bytes.len() as u32;
            unsafe { sig(&bytes[0], len) }
        }

        #[link(wasm_import_module = "canon")]
        extern "C" {
            fn sig(msg: &u8, len: u32);
        }

        use core::fmt::{self, Write};
        use core::panic::PanicInfo;

        impl Write for PanicMsg {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                let bytes = s.as_bytes();
                let len = bytes.len();
                self.buf[self.ofs..self.ofs + len].copy_from_slice(bytes);
                self.ofs += len;
                Ok(())
            }
        }

        struct PanicMsg {
            ofs: usize,
            buf: [u8; 1024],
        }

        impl AsRef<str> for PanicMsg {
            fn as_ref(&self) -> &str {
                core::str::from_utf8(&self.buf[0..self.ofs])
                    .unwrap_or("PanicMsg.as_ref failed.")
            }
        }

        #[panic_handler]
        fn panic(info: &PanicInfo) -> ! {
            let mut msg = PanicMsg {
                ofs: 0,
                buf: [0u8; 1024],
            };

            writeln!(msg, "{}", info).ok();

            signal(msg.as_ref());

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
    }

    // transactions
    type TransactionIndex = u16;

    impl Counter {
        pub fn adjust(by: i32) -> Transaction<(TransactionIndex, i32), ()> {
            Transaction::new((2, by))
        }
    }
}
