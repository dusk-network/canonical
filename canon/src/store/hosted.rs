// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::marker::PhantomData;

use crate::canon::{Canon, InvalidEncoding};
use crate::store::{Sink, Source, Store};

// We set the buffer size to 4kib for now, subject to change.
const BUF_SIZE: usize = 1024 * 4;

static mut BUF: [u8; BUF_SIZE] = [0; BUF_SIZE];

/// Store usable across ffi-boundraries
#[derive(Clone, Copy, Default, Debug)]
pub struct BridgeStore;

impl BridgeStore {
    /// Create a new bridge store
    pub fn new() -> Self {
        BridgeStore {
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BridgeCanonError {
    InvalidEncoding,
}

impl<S: Store> Canon<S> for BridgeCanonError {
    fn write(&self, _sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        Ok(())
    }

    fn read(_source: &mut impl Source<S>) -> Result<Self, S::Error> {
        Ok(BridgeCanonError::InvalidEncoding)
    }

    fn encoded_len(&self) -> usize {
        0
    }
}

impl From<InvalidEncoding> for BridgeCanonError {
    fn from(_: InvalidEncoding) -> Self {
        BridgeCanonError::InvalidEncoding
    }
}

impl Store for BridgeStore {
    type Error = InvalidEncoding;

    fn fetch(
        &self,
        id: &Self::Ident,
        into: &mut [u8],
    ) -> Result<(), Self::Error> {
        unsafe {
            let slice = id.as_ref();
            let id_len = slice.len();
            // first copy the id into the buffer
            into[0..id_len].copy_from_slice(slice);
            get(&mut into[0]);
            Ok(())
        }
    }

    fn get<T: Canon<Self>>(&self, id: &Self::Ident) -> Result<T, Self::Error> {
        unsafe {
            let slice = id.as_ref();
            let id_len = slice.len();
            BUF[0..id_len].copy_from_slice(slice);
            get(&mut BUF[0]);
            // get has written T into the buffer
            let mut source = ByteSource::new(&BUF[..], self);
            Canon::<Self>::read(&mut source)
        }
    }

    fn put<T: Canon<Self>>(&self, t: &T) -> Result<Self::Ident, Self::Error> {
        unsafe {
            let len = t.encoded_len();
            let mut sink = ByteSink::new(&mut BUF, self);
            Canon::<Self>::write(t, &mut sink)?;
            let mut id = Self::Ident::default();
            put(&mut BUF[0], len, &mut id.as_mut()[0]);
            Ok(id)
        }
    }

    fn put_raw(&self, bytes: &[u8]) -> Result<Self::Ident, Self::Error> {
        unsafe {
            let mut id = Self::Ident::default();
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
