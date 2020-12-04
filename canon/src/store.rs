// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::hash::Hash;

use crate::{Canon, DrySink, InvalidEncoding};

/// Trait for types that takes bytes and produces an identifier
pub trait IdBuilder<I>: Default {
    /// Write bytes to the id state
    fn write_bytes(&mut self, bytes: &[u8]);
    /// Consume builder and create an Ident
    fn fin(self) -> I;
}

/// Restrictions on types acting as identifiers
pub trait Ident:
    'static
    + Default
    + AsRef<[u8]>
    + AsMut<[u8]>
    + Clone
    + Eq
    + Copy
    + Hash
    + core::fmt::Debug
{
    /// Takes bytes to produce an identifier
    type Builder: IdBuilder<Self>;
}

/// Trait to implement writing bytes to an underlying storage
pub trait Sink<S: Store> {
    /// Copy bytes from a slice into the `Sink`
    fn copy_bytes(&mut self, bytes: &[u8]);
    /// Recursively create another sink for storing children
    fn recur<T: Canon<S>>(&self, t: &T) -> Result<S::Ident, S::Error>;
    /// Consume the sink and return the ident of written data
    fn fin(self) -> Result<S::Ident, S::Error>;
}

/// Trait to implement reading bytes from an underlying storage
pub trait Source<S> {
    /// Request n bytes from the sink to be read
    fn read_bytes(&mut self, n: usize) -> &[u8];
    /// Returns a reference to the Store associated with the source
    fn store(&self) -> &S;
}

/// The main trait for storing/transmitting data, in the case of a wasm
/// environment, this is generally implemented with host calls
pub trait Store: 'static + Clone + Default {
    /// The identifier used for allocations
    type Ident: Ident;
    /// The error the store can emit
    type Error: From<InvalidEncoding> + core::fmt::Debug;

    /// Write bytes associated with `Ident`
    fn fetch(
        &self,
        id: &Self::Ident,
        into: &mut [u8],
    ) -> Result<(), Self::Error>;

    /// Get a value from storage, given an identifier
    fn get<T: Canon<Self>>(&self, id: &Self::Ident) -> Result<T, Self::Error>;

    /// Encode a value into the store
    fn put<T: Canon<Self>>(&self, t: &T) -> Result<Self::Ident, Self::Error>;

    /// Put raw bytes in store
    fn put_raw(&self, bytes: &[u8]) -> Result<Self::Ident, Self::Error>;

    /// Calculate the Identifier of a type without storing it
    fn ident<T: Canon<Self>>(t: &T) -> Self::Ident {
        let mut sink = DrySink::new();
        t.write(&mut sink).expect("Drysink cannot fail");
        sink.fin().expect("Drysink cannot fail")
    }
}

impl<S> Canon<S> for S
where
    S: Store,
{
    fn write(&self, _: &mut impl Sink<S>) -> Result<(), S::Error> {
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        Ok(source.store().clone())
    }

    fn encoded_len(&self) -> usize {
        0
    }
}

/// A sink over a slice of bytes
pub struct ByteSink<'a, S: Store> {
    bytes: &'a mut [u8],
    offset: usize,
    store: S,
    builder: <S::Ident as Ident>::Builder,
}

impl<'a, S> ByteSink<'a, S>
where
    S: Store,
{
    /// Creates a new sink reading from bytes
    pub fn new(bytes: &'a mut [u8], store: S) -> Self {
        ByteSink {
            bytes,
            store,
            offset: 0,
            builder: Default::default(),
        }
    }
}

impl<'a, S> Sink<S> for ByteSink<'a, S>
where
    S: Store,
{
    fn copy_bytes(&mut self, bytes: &[u8]) {
        self.builder.write_bytes(bytes);
        let len = bytes.len();
        self.bytes[self.offset..self.offset + len].copy_from_slice(bytes);
        self.offset += len;
    }

    fn recur<T: Canon<S>>(&self, t: &T) -> Result<S::Ident, S::Error> {
        self.store.put(t)
    }

    fn fin(self) -> Result<S::Ident, S::Error> {
        Ok(self.builder.fin())
    }
}

/// A sink over a slice of bytes
pub struct ByteSource<'a, S> {
    bytes: &'a [u8],
    offset: usize,
    store: S,
}

impl<'a, S> ByteSource<'a, S> {
    /// Creates a new sink reading from bytes
    pub fn new(bytes: &'a [u8], store: S) -> Self {
        ByteSource {
            bytes,
            store,
            offset: 0,
        }
    }
}

impl<'a, S> Source<S> for ByteSource<'a, S>
where
    S: Store,
{
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let old_offset = self.offset;
        self.offset += n;
        &self.bytes[old_offset..old_offset + n]
    }

    fn store(&self) -> &S {
        &self.store
    }
}
