// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use blake2b_simd::Params;
use lazy_static::lazy_static;
use parking_lot::RwLock;

use std::collections::HashMap;
use std::sync::Arc;

use crate::id::IdHash;
use crate::CanonError;

lazy_static! {
    static ref STATIC_MAP: Arc<RwLock<HashMap<IdHash, Vec<u8>>>> =
        Arc::new(RwLock::new(HashMap::new()));
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
        debug_assert!(bytes.len() > core::mem::size_of::<IdHash>());
        let hash = Self::hash(bytes);
        STATIC_MAP.write().insert(hash, Vec::from(bytes));
        hash
    }

    pub fn hash(bytes: &[u8]) -> IdHash {
        let mut state = Params::new().hash_length(32).to_state();
        state.update(bytes);

        let mut buf = [0u8; 32];
        buf.copy_from_slice(state.finalize().as_ref());
        buf
    }
}
