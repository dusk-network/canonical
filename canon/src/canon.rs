// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use core::fmt::Debug;

use crate::store::{Sink, Source, Store};

/// The main crate error type.
#[derive(Debug)]
pub enum CanonError {
    /// The data read was invalid
    InvalidData,
    /// The value was missing in a store
    MissingValue,
}

/// Trait to read/write values as bytes
pub trait Canon<S: Store>: Sized {
    /// Write the value as bytes to a `Sink`
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), CanonError>;
    /// Read the value from bytes in a `Source`
    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError>;
    /// Returns the number of bytes needed to encode this value
    fn encoded_len(&self) -> usize;
}
