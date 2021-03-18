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

use crate::{Canon, CanonError, Id, Sink, Source};

struct InMemoryMap(HashMap<Id, Vec<u8>>);

impl InMemoryMap {
    fn new() -> Self {
        InMemoryMap(HashMap::new())
    }

    fn insert(&mut self, id: Id, bytes: Vec<u8>) {
        self.0.insert(id, bytes);
    }

    fn get(&self, id: &Id) -> Option<&[u8]> {
        self.0.get(id).map(AsRef::as_ref)
    }
}

lazy_static! {
    static ref STATIC_MAP: Arc<RwLock<InMemoryMap>> =
        Arc::new(RwLock::new(InMemoryMap::new()));
}

pub(crate) struct HostStore;

impl HostStore {
    pub(crate) fn fetch(_id: &Id, _into: &mut [u8]) -> Result<(), CanonError> {
        todo!("a");
    }

    pub(crate) fn put<T: Canon>(t: &T) -> Id {
        let len = t.encoded_len();
        let mut vec = Vec::with_capacity(len);
        vec.resize_with(len, || 0);
        let mut sink = Sink::new(&mut vec[..]);
        t.encode(&mut sink);
        let id = sink.fin();
        STATIC_MAP.write().insert(id, vec);
        id
    }

    pub(crate) fn put_raw(bytes: &[u8]) -> Id {
        let len = bytes.len();
        let mut vec = Vec::with_capacity(len);
        vec.resize_with(len, || 0);
        let mut sink = Sink::new(&mut vec[..]);
        sink.copy_bytes(bytes);
        let id = sink.fin();
        STATIC_MAP.write().insert(id, vec);
        id
    }

    pub(crate) fn get<T: Canon>(id: &Id) -> Result<T, CanonError> {
        match STATIC_MAP.read().get(id) {
            Some(bytes) => {
                let mut source = Source::new(bytes);
                T::decode(&mut source)
            }
            None => Err(CanonError::NotFound),
        }
    }

    pub(crate) fn id<T: Canon>(t: &T) -> Id {
        // Same as put, just not storing anything
        let len = t.encoded_len();
        let mut vec = Vec::with_capacity(len);
        vec.resize_with(len, || 0);
        let mut sink = Sink::new(&mut vec[..]);
        t.encode(&mut sink);
        let id = sink.fin();
        id
    }

    pub fn hash(bytes: &[u8]) -> [u8; 32] {
        let mut state = Params::new().hash_length(32).to_state();
        state.update(&bytes[..]);

        let mut buf = [0u8; 32];
        buf.copy_from_slice(state.finalize().as_ref());
        buf
    }
}
