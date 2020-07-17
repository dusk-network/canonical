use crate::canon::{Canon, CanonError, ConstantLength};

/// Restrictions on types acting as identifiers
pub trait Ident: ConstantLength + Default + AsRef<[u8]> + AsMut<[u8]> {}

impl<T> Ident for T where T: ConstantLength + Default + AsRef<[u8]> + AsMut<[u8]>
{}

/// Trait to implement writing bytes to an underlying storage
pub trait Sink {
    /// Request n bytes to be written
    fn write_bytes(&mut self, n: usize) -> &mut [u8];
    /// Copy bytes from a slice into the `Sink`
    fn copy_bytes(&mut self, bytes: &[u8]);
}

/// Trait to implement reading bytes from an underlying storage
pub trait Source {
    /// Request n bytes from the sink to be read
    fn read_bytes(&mut self, n: usize) -> &[u8];
}

/// The main trait for storing data, in the case of a wasm environment,
/// this is generally implemented with host calls
pub trait Store {
    /// The identifier used for allocations
    type Ident: Ident;
    /// The error the store can emit
    type Error: From<CanonError>;

    /// Put a value into storage, returning an identifier
    fn put<T: Canon>(t: &mut T) -> Result<Self::Ident, Self::Error>;
    /// Get a value from storag, given an identifier
    fn get<T: Canon>(id: &Self::Ident) -> Result<T, Self::Error>;
}

/// Hack to allow the derive macro to assume stores are `Canon`
#[doc(hidden)]
impl<S> Canon for S
where
    S: Store,
{
    fn write(&self, _: &mut impl Sink) {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }

    fn read(_: &mut impl Source) -> Result<Self, CanonError> {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }
}

#[doc(hidden)]
impl<S> ConstantLength for S
where
    S: Store,
{
    const LEN: usize = 0;
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

impl Source for &[u8] {
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let slice = core::mem::replace(self, &mut []);
        let (a, b) = slice.split_at(n);
        *self = b;
        a
    }
}
