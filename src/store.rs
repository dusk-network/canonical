use core::marker::PhantomData;

use crate::canon::{Canon, CanonError};

/// Restrictions on types acting as identifiers
pub trait Ident: Default + AsRef<[u8]> + AsMut<[u8]> + Clone {}
impl<T> Ident for T where T: Default + AsRef<[u8]> + AsMut<[u8]> + Clone {}

/// Trait to implement writing bytes to an underlying storage
pub trait Sink {
    /// Request n bytes to be written
    fn write_bytes(&mut self, n: usize) -> &mut [u8];
    /// Copy bytes from a slice into the `Sink`
    fn copy_bytes(&mut self, bytes: &[u8]);
}

/// Trait to implement reading bytes from an underlying storage
pub trait Source<S> {
    /// Request n bytes from the sink to be read
    fn read_bytes(&mut self, n: usize) -> &[u8];
    /// Returns a copy of the Store associated with the source
    fn store(&self) -> S;
}

/// The main trait for storing/transmitting data, in the case of a wasm environment,
/// this is generally implemented with host calls
pub trait Store: Clone {
    /// The identifier used for allocations
    type Ident: Ident;
    /// The error the store can emit
    type Error: From<CanonError>;

    /// Put a value into storage, returning an identifier
    fn put<T: Canon<Self>>(
        &mut self,
        t: &mut T,
    ) -> Result<Self::Ident, Self::Error>;
    /// Get a value from storag, given an identifier
    fn get<T: Canon<Self>>(&self, id: &Self::Ident) -> Result<T, Self::Error>;

    /// Create a snapshot from a value
    fn snapshot<T: Canon<Self>>(
        &mut self,
        t: &mut T,
    ) -> Result<Snapshot<T, Self>, Self::Error> {
        let id = self.put(t)?;
        Ok(Snapshot {
            id,
            store: self.clone(),
            _marker: PhantomData,
        })
    }

    fn singleton() -> Self;
}

/// A snapshot of a host-alloctated value.
pub struct Snapshot<T: ?Sized, S: Store> {
    id: S::Ident,
    store: S,
    _marker: PhantomData<T>,
}

impl<T, S> Snapshot<T, S>
where
    S: Store,
    T: Canon<S>,
{
    /// Extracts the value from the snapshot
    pub fn restore(&self) -> Result<T, S::Error> {
        self.store.get::<T>(&self.id)
    }
}

/// Hack to allow the derive macro to assume stores are `Canon`
#[doc(hidden)]
impl<S> Canon<S> for S
where
    S: Store,
{
    fn write(&mut self, _: &mut impl Sink) {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }

    fn read(_: &mut impl Source<S>) -> Result<Self, CanonError> {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }

    fn encoded_len(&self) -> usize {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }
}

impl Sink for &mut [u8] {
    fn write_bytes(&mut self, n: usize) -> &mut [u8] {
        let slice = core::mem::replace(self, &mut []);
        let (a, b) = slice.split_at_mut(n);
        *self = b;
        a
    }

    fn copy_bytes(&mut self, bytes: &[u8]) {
        let n = bytes.len();
        let slice = core::mem::replace(self, &mut []);
        let (a, b) = slice.split_at_mut(n);
        *self = b;
        a.copy_from_slice(bytes)
    }
}

impl<S> Source<S> for &[u8] {
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let slice = core::mem::replace(self, &[]);
        let (a, b) = slice.split_at(n);
        *self = b;
        a
    }

    fn store(&self) -> S {
        panic!("Attempt to get source from slice")
    }
}

#[derive(Clone)]
/// A store that does not store anything
pub struct VoidStore;

impl Store for VoidStore {
    type Ident = [u8; 0];
    type Error = CanonError;

    fn put<T: Canon<Self>>(
        &mut self,
        _: &mut T,
    ) -> Result<Self::Ident, Self::Error> {
        Ok([])
    }

    fn get<T: Canon<Self>>(&self, _: &Self::Ident) -> Result<T, Self::Error> {
        Err(CanonError::MissingValue)
    }

    fn singleton() -> Self {
        VoidStore
    }
}
