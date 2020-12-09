// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(lang_items, never_type)]

use canonical_derive::Canon;

#[derive(Clone, Canon, Debug)]
pub struct Panico;

// query ids
pub const PANIC_A: u8 = 0;
pub const PANIC_B: u8 = 1;

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSource, Canon, Id32, Store};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl Panico {
        pub fn panic_a(&self) {
            panic!("let's panic!");
        }

        pub fn panic_b(&self) {
            panic!("let's panic differently!");
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let store = BS::default();
        let mut source = ByteSource::new(&bytes[..], &store);

        // no-op reading a unit struct;
        let slf = Panico::read(&mut source)?;

        let tag = u8::read(&mut source)?;
        match tag {
            0 => slf.panic_a(),
            _ => slf.panic_b(),
        }
        Ok(())
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        let _ = query(bytes);
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
    use canonical_host::Query;

    impl Panico {
        pub fn panic_a() -> Query<Self, (), !, PANIC_A> {
            Query::new(())
        }

        pub fn panic_b() -> Query<Self, (), !, PANIC_B> {
            Query::new(())
        }
    }
}
