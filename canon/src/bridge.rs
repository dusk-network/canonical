// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::canon::{Canon, CanonError};
use crate::id::Id;
use crate::store::{Sink, Source};

// We set the buffer size to 32kib for now, subject to change.
const BUF_SIZE: usize = 1024 * 32;

#[allow(unused)] // FIXME
static mut BUF: [u8; BUF_SIZE] = [0; BUF_SIZE];

/// Store usable across ffi-boundraries
#[derive(Clone, Copy, Default, Debug)]
struct BridgeStore;

impl BridgeStore {
    /// Create a new bridge store
    #[allow(unused)] // FIXME
    pub fn new() -> Self {
        BridgeStore
    }

    #[allow(unused)] // FIXME
    fn fetch(&self, id: &Id, into: &mut [u8]) -> Result<(), CanonError> {
        unsafe {
            let slice = id.as_ref();
            let id_len = slice.len();
            // first copy the id into the buffer
            into[0..id_len].copy_from_slice(slice);
            get(&mut into[0]);
            Ok(())
        }
    }

    #[allow(unused)] // FIXME
    fn get<T: Canon>(&self, id: &Id) -> Result<T, CanonError> {
        unsafe {
            let slice = id.as_ref();
            let id_len = slice.len();
            BUF[0..id_len].copy_from_slice(slice);
            get(&mut BUF[0]);
            // get has written T into the buffer
            let mut source = Source::new(&BUF[..]);
            Canon::read(&mut source)
        }
    }

    #[allow(unused)] // FIXME
    fn put<T: Canon>(&self, t: &T) -> Result<Id, CanonError> {
        unsafe {
            let len = t.encoded_len();
            let mut sink = Sink::new(&mut BUF);
            Canon::write(t, &mut sink);
            let mut id = Id::default();
            put(&mut BUF[0], len, &mut id.as_mut()[0]);
            Ok(id)
        }
    }

    #[allow(unused)] // FIXME
    fn put_raw(&self, bytes: &[u8]) -> Result<Id, CanonError> {
        unsafe {
            let mut id = Id::default();
            let len = bytes.len();
            BUF[0..len].copy_from_slice(bytes);
            put(&mut BUF[0], len, &mut id.as_mut()[0]);
            Ok(id)
        }
    }
}

#[link(wasm_import_module = "canon")]
extern "C" {
    pub fn put(buf: &mut u8, len: usize, ret: &mut u8);
    pub fn get(buf: &mut u8);
}
