// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use blake2b_simd::Params;

use std::cell::RefCell;
use std::collections::HashMap;

use crate::canon::CanonError;
use crate::id::{Id, IdHash};

thread_local! {
    pub static STATIC_MAP: RefCell<HashMap<IdHash, Vec<u8>>> =
    RefCell::new(HashMap::new())
}

pub(crate) struct HostStore;

impl HostStore {
    pub(crate) fn get(
        hash: &IdHash,
        into: &mut [u8],
    ) -> Result<(), CanonError> {
        match STATIC_MAP.with(|m| {
            m.borrow().get(hash).map(|vec| into.copy_from_slice(&vec))
        }) {
            Some(()) => Ok(()),
            None => Err(CanonError::NotFound),
        }
    }

    pub(crate) fn put(bytes: &[u8]) -> IdHash {
        debug_assert!(bytes.len() > core::mem::size_of::<IdHash>());
        let hash = Self::hash(bytes);
        STATIC_MAP.with(|m| m.borrow_mut().insert(hash, Vec::from(bytes)));
        hash
    }

    pub(crate) fn hash(bytes: &[u8]) -> IdHash {
        let mut state = Params::new().hash_length(32).to_state();
        state.update(bytes);

        let mut buf = [0u8; 32];
        buf.copy_from_slice(state.finalize().as_ref());
        buf
    }

    pub(crate) fn promote_bytes(id: &Id) -> Result<Vec<u8>, CanonError> {
        STATIC_MAP.with(|m| {
            if let Some(vec) = m.borrow_mut().remove(&id.hash()) {
                if id.size() == vec.len() {
                    Ok(vec)
                } else {
                    Err(CanonError::InvalidEncoding)
                }
            } else {
                Err(CanonError::NotFound)
            }
        })
    }
}
