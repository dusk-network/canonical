// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

#![cfg_attr(feature = "bridge", no_std)]
#![feature(lang_items)]

use canonical::Canon;
use canonical_derive::Canon;

#[derive(Clone, Canon, Default)]
pub struct Counter {
    value: i32,
}

impl Counter {
    pub fn new(value: i32) -> Self {
        Counter { value }
    }
}

#[cfg(feature = "bridge")]
mod bridge {
    use super::*;

    use canonical::BridgeStore;

    #[no_mangle]
    fn q(bytes: &[u8]) {
        // query is actually a no-op, since the return type
        // is just a newtype
    }

    #[no_mangle]
    fn t(bytes: &mut [u8]) {
        let source = &mut bytes;
        let slf: Counter = Canon::<BridgeStore<[u8; 8]>>::read(&mut source);
        let args: i32 = Canon::<BridgeStore<[u8; 8]>>::read(source);
        slf.adjust(args);
    }

    #[cfg(feature = "bridge")]
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
