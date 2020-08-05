use core::marker::PhantomData;

use crate::canon::{Canon, CanonError};

/// Restrictions on types acting as identifiers
pub trait Ident:
    Default + AsRef<[u8]> + AsMut<[u8]> + Clone + std::fmt::Debug
{
}
impl<T> Ident for T where
    T: Default + AsRef<[u8]> + AsMut<[u8]> + Clone + std::fmt::Debug
{
}

/// Trait to implement writing bytes to an underlying storage
pub trait Sink<S: Store> {
    /// Request n bytes to be written
    fn write_bytes(&mut self, n: usize) -> &mut [u8];
    /// Copy bytes from a slice into the `Sink`
    fn copy_bytes(&mut self, bytes: &[u8]);
    /// Recursively create another sink for storing children
    fn recur(&self) -> Self;
    /// Finish the sink, store the value, and return the identity
    fn fin(self) -> Result<S::Ident, CanonError<S>>;
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
    type Error: core::fmt::Debug;

    /// Put a value into storage, returning an identifier
    fn put<T: Canon<Self>>(
        &self,
        t: &mut T,
    ) -> Result<Self::Ident, CanonError<Self>>;
    /// Get a value from storag, given an identifier
    fn get<T: Canon<Self>>(
        &self,
        id: &Self::Ident,
    ) -> Result<T, CanonError<Self>>;

    /// Create a snapshot from a value
    fn snapshot<T: Canon<Self>>(
        &self,
        t: &mut T,
    ) -> Result<Snapshot<T, Self>, CanonError<Self>> {
        let id = self.put(t)?;
        Ok(Snapshot {
            id,
            store: self.clone(),
            _marker: PhantomData,
        })
    }

    /// Store raw bytes in the store
    fn put_raw(&self, bytes: &[u8]) -> Result<Self::Ident, CanonError<Self>>;

    #[cfg(feature = "bridge")]
    /// Only applicable to the bridge version
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
    pub fn restore(&self) -> Result<T, CanonError<S>> {
        self.store.get::<T>(&self.id)
    }
}

/// Hack to allow the derive macro to assume stores are `Canon`
#[doc(hidden)]
impl<S> Canon<S> for S
where
    S: Store,
{
    fn write(&self, _: &mut impl Sink<S>) -> Result<(), CanonError<S>> {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }

    fn read(_: &mut impl Source<S>) -> Result<Self, CanonError<S>> {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }

    fn encoded_len(&self) -> usize {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }
}

impl<S: Store> Sink<S> for &mut [u8] {
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

    fn recur(&self) -> Self {
        unimplemented!("Non-recursive sink")
    }

    fn fin(self) -> Result<S::Ident, CanonError<S>> {
        unimplemented!("Non-recursive sink")
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
        unimplemented!("Non-recursive source")
    }
}

#[derive(Clone, Debug)]
/// A store that does not store anything
pub struct VoidStore;

impl Store for VoidStore {
    type Ident = [u8; 0];
    type Error = ();

    fn put<T: Canon<Self>>(
        &self,
        _: &mut T,
    ) -> Result<Self::Ident, CanonError<Self>> {
        Ok([])
    }

    fn get<T: Canon<Self>>(
        &self,
        _: &Self::Ident,
    ) -> Result<T, CanonError<Self>> {
        Err(CanonError::MissingValue)
    }

    fn put_raw(&self, _: &[u8]) -> Result<Self::Ident, CanonError<Self>> {
        Ok([])
    }

    #[cfg(feature = "bridge")]
    fn singleton() -> Self {
        VoidStore
    }
}
