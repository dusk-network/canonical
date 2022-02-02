// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use blake2b_simd::Params;

use std::collections::BTreeMap;

use parking_lot::RwLock;

use crate::canon::CanonError;
use crate::id::{Id, IdHash};

lazy_static::lazy_static! {
    pub static ref STATIC_MAP: RwLock<BTreeMap<IdHash, Vec<u8>>> =
        RwLock::new(BTreeMap::new());
}

pub(crate) struct HostStore;

impl HostStore {
    pub(crate) fn get(
        hash: &IdHash,
        into: &mut [u8],
    ) -> Result<(), CanonError> {
        match STATIC_MAP
            .read()
            .get(hash)
            .map(|vec| into.copy_from_slice(&vec))
        {
            Some(()) => Ok(()),
            None => Err(CanonError::NotFound),
        }
    }

    pub(crate) fn put(bytes: &[u8]) -> IdHash {
        // If length is less than that of a hash, this should have been inlined.
        debug_assert!(bytes.len() > core::mem::size_of::<IdHash>());
        let hash = Self::hash(bytes);
        STATIC_MAP.write().insert(hash, Vec::from(bytes));
        hash
    }

    pub(crate) fn hash(bytes: &[u8]) -> IdHash {
        let mut state = Params::new().hash_length(32).to_state();
        state.update(bytes);

        let mut buf = [0u8; 32];
        buf.copy_from_slice(state.finalize().as_ref());
        buf
    }

    pub(crate) fn take_bytes(id: &Id) -> Result<Vec<u8>, CanonError> {
        match STATIC_MAP.write().remove(&id.hash()) {
            Some(vec) if id.size() == vec.len() => Ok(vec),
            Some(_) => Err(CanonError::InvalidEncoding),
            None => Err(CanonError::NotFound),
        }
    }
}
