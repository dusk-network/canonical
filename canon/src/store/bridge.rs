// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

extern crate alloc;

use crate::canon::CanonError;
use crate::id::{Id, IdHash};
use alloc::vec::Vec;

/// Store usable across ffi-boundaries
#[derive(Clone, Copy, Default, Debug)]
pub struct BridgeStore;

impl BridgeStore {
    pub(crate) fn put(bytes: &[u8]) -> IdHash {
        // We only put larger values here
        debug_assert!(bytes.len() > core::mem::size_of::<IdHash>());
        let mut idhash = IdHash::default();
        unsafe {
            put(&bytes[0], bytes.len() as i32, &mut idhash);
        }
        idhash
    }

    pub fn get(hash: &IdHash, into: &mut [u8]) -> Result<(), CanonError> {
        // We assume this to always work for the bridge, by catching the error
        // in the host and aborting before returning.
        let len = into.len();
        unsafe { Ok(get(&hash, &mut into[0], len as i32)) }
    }

    pub fn hash(bytes: &[u8]) -> IdHash {
        let len = bytes.len();
        let ofs = &bytes[0];
        let mut result = IdHash::default();
        unsafe { hash(ofs, len as i32, &mut result) };
        result
    }

    pub fn take_bytes(id: &Id) -> Result<Vec<u8>, CanonError> {
        // No-op in bridge version
        let len = id.size();
        let mut buf = Vec::with_capacity(len);
        buf.resize_with(len, || 0);
        Self::get(&id.hash(), &mut buf[..])?;
        Ok(buf)
    }
}

#[link(wasm_import_module = "canon")]
extern "C" {
    pub fn put(buf: &u8, len: i32, ret_hash: &mut IdHash);
    pub fn get(hash: &IdHash, buf: &mut u8, len: i32);
    pub fn hash(ofs: &u8, len: i32, buf: &mut IdHash);
}
