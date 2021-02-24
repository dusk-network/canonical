// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::store::{Sink, Source};

/// The possible errors when fetching/decoding values from a store
#[derive(Debug)]
pub enum CanonError {
    /// The byte sequence is not a valid representation of the type decoded
    InvalidEncoding,
    /// The instance could not be found in storage
    NotFound,
}

/// Trait to read/write values as bytes
pub trait Canon: Sized + Clone {
    /// Write the value as bytes to a `Sink`
    fn write(&self, sink: &mut Sink);
    /// Read the value from bytes in a `Source`
    fn read(source: &mut Source) -> Result<Self, CanonError>;
    /// Returns the number of bytes needed to encode this value
    fn encoded_len(&self) -> usize;
}
