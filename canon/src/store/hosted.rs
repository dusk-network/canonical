// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::canon::{Canon, CanonError};
use crate::id::Id;
use crate::store::{Sink, Source};

extern crate alloc;

use alloc::vec::Vec;

/// Store usable across ffi-boundraries
#[derive(Clone, Copy, Default, Debug)]
pub struct BridgeStore;

impl BridgeStore {
    fn get<T: Canon>(&self, id: &Id) -> Result<T, CanonError> {
        unsafe {
            let len = id.len();
            let mut buf = Vec::with_capacity();
            buf.resize_with(len, || 0);
            get(&id, &mut buf[0]);
            // get has now written the encoded bytes of T into the buffer
            let mut source = Source::new(&buf[..]);
            T::read(&mut source)
        }
    }

    fn put<T: Canon>(&self, t: &T) {
        unsafe {
            let len = t.encoded_len();
            let mut buf = Vec::with_capacity();
            buf.resize_with(len, || 0);
            let mut sink = Sink::new(&mut buf[..]);
            t.write(&mut sink);
            let mut id = Id::default();
            put(&mut buf[0], len, &mut id.as_mut()[0]);
            Ok(id)
        }
    }

    fn put_raw(&self, bytes: &[u8]) {
        unsafe {
            let mut id = Self::Ident::default();
            let len = bytes.len();

            todo!()
        }
    }
}

#[link(wasm_import_module = "canon")]
extern "C" {
    pub fn put(buf: &mut u8, len: usize, ret: &mut u8);
    pub fn get(buf: &mut u8);
}
