// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::fmt::Debug;

use crate::store::{Sink, Source, Store};

/// The sole error that can be encountered by reading data
#[derive(Debug, Clone)]
pub struct InvalidEncoding;

impl<S: Store> Canon<S> for InvalidEncoding {
    fn write(&self, _sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        Ok(())
    }

    fn read(_source: &mut impl Source<S>) -> Result<Self, S::Error> {
        Ok(InvalidEncoding)
    }

    fn encoded_len(&self) -> usize {
        0
    }
}

/// Trait to read/write values as bytes
pub trait Canon<S: Store>: Sized + Clone {
    /// Write the value as bytes to a `Sink`
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error>;
    /// Read the value from bytes in a `Source`
    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error>;
    /// Returns the number of bytes needed to encode this value
    fn encoded_len(&self) -> usize;
}
