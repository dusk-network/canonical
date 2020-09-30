// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use crate::canon::{Canon, InvalidEncoding};

/// Restrictions on types acting as identifiers
pub trait Ident:
    Default + AsRef<[u8]> + AsMut<[u8]> + Clone + core::fmt::Debug
{
}

impl<T> Ident for T where
    T: Default + AsRef<[u8]> + AsMut<[u8]> + Clone + core::fmt::Debug
{
}

/// Trait to implement writing bytes to an underlying storage
pub trait Sink<S: Store> {
    /// Request n bytes to be written
    fn write_bytes(&mut self, n: usize) -> &mut [u8];
    /// Copy bytes from a slice into the `Sink`
    fn copy_bytes(&mut self, bytes: &[u8]);
    /// Recursively create another sink for storing children
    fn recur<T: Canon<S>>(&mut self, t: &T) -> Result<S::Ident, S::Error>;
}

/// Trait to implement reading bytes from an underlying storage
pub trait Source<S> {
    /// Request n bytes from the sink to be read
    fn read_bytes(&mut self, n: usize) -> &[u8];
    /// Returns a reference to the Store associated with the source
    fn store(&self) -> &S;
}

/// The main trait for storing/transmitting data, in the case of a wasm environment,
/// this is generally implemented with host calls
pub trait Store: Clone {
    /// The identifier used for allocations
    type Ident: Ident;
    /// The error the store can emit
    type Error: From<InvalidEncoding>;

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

    /// For hosted environments, get a reference to the current store
    #[cfg(feature = "hosted")]
    fn singleton() -> Self;
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
pub struct ByteSink<'a, S> {
    bytes: &'a mut [u8],
    offset: usize,
    #[allow(unused)]
    store: S,
}

impl<'a, S> ByteSink<'a, S> {
    /// Creates a new sink reading from bytes
    pub fn new(bytes: &'a mut [u8], store: S) -> Self {
        ByteSink {
            bytes,
            store,
            offset: 0,
        }
    }
}

impl<'a, S> Sink<S> for ByteSink<'a, S>
where
    S: Store,
{
    fn write_bytes(&mut self, _n: usize) -> &mut [u8] {
        unimplemented!("döden");
    }

    fn copy_bytes(&mut self, bytes: &[u8]) {
        let len = bytes.len();
        self.bytes[self.offset..self.offset + len].copy_from_slice(bytes);
        self.offset += len;
    }

    fn recur<T: Canon<S>>(&mut self, t: &T) -> Result<S::Ident, S::Error> {
        self.store.put(t)
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
