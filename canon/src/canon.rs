use core::fmt::Debug;

use crate::store::{Sink, Source, Store};

/// The main crate error type.
#[derive(Debug)]
pub enum CanonError<E> {
    /// The data read was invalid
    InvalidData,
    /// The value was missing in a store
    MissingValue,
    /// Error emenating from the underlying store
    StoreError(E),
}

/// Trait to read/write values as bytes
pub trait Canon<S: Store>: Sized + Clone {
    /// Write the value as bytes to a `Sink`
    fn write(
        &self,
        sink: &mut impl Sink<S>,
    ) -> Result<(), CanonError<S::Error>>;
    /// Read the value from bytes in a `Source`
    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>>;
    /// Returns the number of bytes needed to encode this value
    fn encoded_len(&self) -> usize;
}