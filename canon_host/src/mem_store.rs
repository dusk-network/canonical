// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::collections::hash_map::{DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use parking_lot::RwLock;

use canonical::{ByteSink, Canon, InvalidEncoding, Sink, Source, Store};
use canonical_derive::Canon;

#[derive(Default, Debug)]
struct MemStoreInner {
    map: HashMap<[u8; 8], Vec<u8>>,
    head: usize,
}

#[derive(Default, Debug, Clone)]
pub struct MemStore(Arc<RwLock<MemStoreInner>>);

impl MemStore {
    pub fn new() -> Self {
        Default::default()
    }
}

struct MemSink<S> {
    bytes: Vec<u8>,
    store: S,
}

struct MemSource<'a, S> {
    bytes: &'a [u8],
    offset: usize,
    store: S,
}

#[derive(Canon, Debug)]
pub enum MemError {
    MissingValue,
    InvalidEncoding,
}

impl From<InvalidEncoding> for MemError {
    fn from(_: InvalidEncoding) -> Self {
        MemError::InvalidEncoding
    }
}

fn hash_of(bytes: &[u8]) -> [u8; 8] {
    let mut hasher = DefaultHasher::new();
    bytes[..].hash(&mut hasher);
    hasher.finish().to_be_bytes()
}

impl Store for MemStore {
    type Ident = [u8; 8];
    type Error = MemError;

    fn fetch(
        &self,
        id: &Self::Ident,
        into: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.0
            .read()
            .map
            .get(id)
            .map(|bytes| {
                let len = bytes.len();
                into[0..len].copy_from_slice(&bytes[..]);
                Ok(())
            })
            .unwrap_or(Err(MemError::MissingValue))
    }

    fn get<T: Canon<Self>>(&self, id: &Self::Ident) -> Result<T, Self::Error> {
        self.0
            .read()
            .map
            .get(id)
            .map(|bytes| {
                let mut source = MemSource {
                    bytes,
                    offset: 0,
                    store: self.clone(),
                };
                T::read(&mut source)
            })
            .unwrap_or_else(|| Err(MemError::MissingValue))
    }

    fn put<T: Canon<Self>>(&self, t: &T) -> Result<Self::Ident, Self::Error> {
        let len = t.encoded_len();
        let mut bytes = Vec::with_capacity(len);
        bytes.resize_with(len, || 0);

        let mut sink = ByteSink::new(&mut bytes, self.clone());
        Canon::<Self>::write(t, &mut sink)?;

        debug_assert!(bytes[..].len() == len);

        let hash = hash_of(&bytes[..]);

        self.0.write().map.insert(hash, bytes);
        Ok(hash)
    }

    fn put_raw(&self, bytes: &[u8]) -> Result<Self::Ident, Self::Error> {
        let hash = hash_of(bytes);
        self.0.write().map.insert(hash, bytes.to_vec());
        Ok(hash)
    }
}

impl<S: Store> Sink<S> for MemSink<S> {
    fn write_bytes(&mut self, n: usize) -> &mut [u8] {
        let ofs = self.bytes.len();
        self.bytes.resize_with(n, || 0);
        &mut self.bytes[ofs..]
    }

    fn copy_bytes(&mut self, bytes: &[u8]) {
        let ofs = self.bytes.len();
        self.bytes.resize_with(ofs + bytes.len(), || 0);
        self.bytes[ofs..].clone_from_slice(bytes)
    }

    fn recur<T: Canon<S>>(&mut self, t: &T) -> Result<S::Ident, S::Error> {
        self.store.put(t)
    }
}

impl<'a, S> Source<S> for MemSource<'a, S>
where
    S: Store,
{
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let ofs = self.offset;
        self.offset += n;
        &self.bytes[ofs..self.offset]
    }

    fn store(&self) -> &S {
        &self.store
    }
}
