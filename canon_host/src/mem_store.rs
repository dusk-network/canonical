// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use parking_lot::RwLock;

use canonical::{
    ByteSink, Canon, DrySink, Id32, InvalidEncoding, Sink, Source, Store,
};
use canonical_derive::Canon;

use crate::wasm::Signal;

#[derive(Default, Debug)]
struct MemStoreInner(HashMap<Id32, Vec<u8>>);

/// An in-memory store implemented with a hashmap
#[derive(Default, Debug, Clone)]
pub struct MemStore(Arc<RwLock<MemStoreInner>>);

impl MemStore {
    /// Create a new MemStore
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

#[derive(Canon, Debug, Clone, PartialEq)]
/// Errors that can happen using the MemStore.
pub enum MemError {
    /// Value missing in the store
    MissingValue,
    /// Invalid data
    InvalidEncoding,
    /// Signal thrown by module
    Signal(Signal),
}

impl From<wasmi::Error> for MemError {
    fn from(err: wasmi::Error) -> MemError {
        MemError::Signal(err.into())
    }
}

impl fmt::Display for MemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingValue => write!(f, "Missing Value"),
            Self::InvalidEncoding => write!(f, "InvalidEncoding"),
            Self::Signal(msg) => write!(f, "{}", msg),
        }
    }
}

impl wasmi::HostError for MemError {}

impl From<InvalidEncoding> for MemError {
    fn from(_: InvalidEncoding) -> Self {
        MemError::InvalidEncoding
    }
}

impl From<Signal> for MemError {
    fn from(signal: Signal) -> Self {
        MemError::Signal(signal)
    }
}

impl Store for MemStore {
    type Ident = Id32;
    type Error = MemError;

    fn fetch(
        &self,
        id: &Self::Ident,
        into: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.0
            .read()
            .0
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
            .0
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
        let ident = sink.fin()?;

        self.0.write().0.insert(ident, bytes);
        Ok(ident)
    }

    fn put_raw(&self, bytes: &[u8]) -> Result<Self::Ident, Self::Error> {
        let mut sink = DrySink::<Self>::new();
        sink.copy_bytes(bytes);
        let ident = sink.fin()?;
        self.0.write().0.insert(ident, bytes.to_vec());
        Ok(ident)
    }
}

impl<S: Store> Sink<S> for MemSink<S> {
    fn copy_bytes(&mut self, bytes: &[u8]) {
        let ofs = self.bytes.len();
        self.bytes.resize_with(ofs + bytes.len(), || 0);
        self.bytes[ofs..].clone_from_slice(bytes)
    }

    fn recur<T: Canon<S>>(&self, t: &T) -> Result<S::Ident, S::Error> {
        self.store.put(t)
    }

    fn fin(self) -> Result<S::Ident, S::Error> {
        self.store.put_raw(&self.bytes[..])
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
