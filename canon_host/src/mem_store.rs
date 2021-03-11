// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use canonical::{Canon, CanonError, Id, IdBuilder, Sink, Source};

#[derive(Default, Debug)]
struct MemStoreInner(HashMap<Id, Vec<u8>>);

/// An in-memory store implemented with a hashmap
#[derive(Debug, Clone)]
pub struct MemStore(Arc<RwLock<MemStoreInner>>);

impl MemStore {
    fn fetch(&self, id: &Id, into: &mut [u8]) -> Result<(), CanonError> {
        self.0
            .read()
            .0
            .get(id)
            .map(|bytes| {
                let len = bytes.len();
                into[0..len].copy_from_slice(&bytes[..]);
                Ok(())
            })
            .unwrap_or(Err(CanonError::NotFound))
    }

    fn get<T: Canon>(&self, id: &Id) -> Result<T, CanonError> {
        self.0
            .read()
            .0
            .get(id)
            .map(|bytes| {
                let mut source = Source::new(bytes);
                T::read(&mut source)
            })
            .unwrap_or_else(|| Err(CanonError::NotFound))
    }

    fn put<T: Canon>(&self, t: &T) -> Result<Id, CanonError> {
        let len = t.encoded_len();
        let mut bytes = Vec::with_capacity(len);
        bytes.resize_with(len, || 0);

        let mut sink = Sink::new(&mut bytes);
        Canon::write(t, &mut sink);
        let ident = sink.fin();

        self.0.write().0.insert(ident, bytes);
        Ok(ident)
    }

    fn put_raw(&self, bytes: &[u8]) -> Result<Id, CanonError> {
        let mut builder = IdBuilder::new();
        builder.write_bytes(bytes);
        let ident = builder.fin();
        self.0.write().0.insert(ident, bytes.to_vec());
        Ok(ident)
    }
}
