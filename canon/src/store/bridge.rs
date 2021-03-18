// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

extern crate alloc;

use alloc::vec::Vec;

use crate::canon::{Canon, CanonError};
use crate::id::Id;
use crate::store::{Sink, Source};

#[thread_local]
static mut BUFFER: Vec<u8> = Vec::new();

/// Store usable across ffi-boundraries
#[derive(Clone, Copy, Default, Debug)]
pub struct Store;

impl Store {
    pub fn get<T: Canon>(id: &Id) -> Result<T, CanonError> {
        unsafe {
            let len = id.size();
            // ensure we have enough space in our buffer
            // NB: this might reallocate on growing, but never actually
            // shrinks the capacity of the buffer
            BUFFER.resize_with(len, || 0);
            if !get(id, &mut BUFFER[0]) {
                return Err(CanonError::NotFound);
            }
            // get has now written the encoded bytes of T into the buffer
            let mut source = Source::new(&BUFFER[..]);
            T::decode(&mut source)
        }
    }

    pub fn put<T: Canon>(t: &T) -> Id {
        unsafe {
            let len = t.encoded_len();
            // ensure we have enough space in our buffer
            BUFFER.resize_with(len, || 0);

            let mut sink = Sink::new(&mut BUFFER[..]);
            t.encode(&mut sink);
            let mut id = Id::default();
            put(&mut BUFFER[0], len as i32, &mut id);
            id
        }
    }

    pub(crate) fn put_raw(bytes: &[u8]) -> Id {
        unsafe {
            let mut id = Id::default();
            let len = bytes.len();
            BUFFER[0..len].copy_from_slice(bytes);
            put(&mut BUFFER[0], len as i32, &mut id);
            id
        }
    }

    pub fn fetch(id: &Id, into: &mut [u8]) -> Result<(), CanonError> {
        unsafe {
            if !get(&id, &mut into[0]) {
                Err(CanonError::NotFound)
            } else {
                Ok(())
            }
        }
    }

    pub fn id<T: Canon>(_t: &T) -> Id {
        todo!()
    }

    pub fn hash(bytes: &[u8]) -> [u8; 32] {
        let mut buf = [0u8; 32];
        unsafe { hash(&bytes[0], bytes.len() as i32, &mut buf) }
        buf
    }
}

#[link(wasm_import_module = "canon")]
extern "C" {
    pub fn put(buf: &mut u8, len: i32, ret_id: &mut Id);
    pub fn get(id: &Id, buf: &mut u8) -> bool;
    pub fn hash(bytes: &u8, len: i32, buf: &mut [u8; 32]);
}