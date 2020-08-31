// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

#![cfg_attr(feature = "bridge", no_std)]
#![feature(lang_items)]

use canonical::{BridgeStore, Canon};
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

#[no_mangle]
fn q(self_bytes: &[u8], _query: &[u8], ret: &mut [u8]) {
    let self_instance =
        <Counter as Canon<BridgeStore<[u8; 8]>>>::read(&mut &self_bytes[..])
            .unwrap();
    let return_value = self_instance.value;
    Canon::<BridgeStore<[u8; 8]>>::write(&return_value, &mut &mut ret[..])
        .unwrap();
}

#[no_mangle]
fn t(slf: &mut Counter, args: &i32, _ret: &mut i32) {
    slf.value += args;
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
