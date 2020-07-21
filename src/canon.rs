use crate::store::{Sink, Source};

/// The main crate error type.
#[derive(Debug)]
pub enum CanonError {
    /// The data read was invalid
    InvalidData,
    /// The value was missing in a store
    MissingValue,
}

/// Trait to read/write values as bytes
pub trait Canon: Sized {
    /// Write the value as bytes to a `Sink`
    fn write(&self, sink: &mut impl Sink);
    /// Read the value from bytes in a `Source`
    fn read(store: &mut impl Source) -> Result<Self, CanonError>;
    /// Returns the number of bytes needed to encode this value
    fn encoded_len(&self) -> usize;
}
