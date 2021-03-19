// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Canon, CanonError, Id};

/// The singleton responsible for saving and restoring values
pub struct VoidStore;

impl VoidStore {
    pub(crate) fn get<T: Canon>(_id: &Id) -> Result<T, CanonError> {
        Err(CanonError::NotFound)
    }

    pub(crate) fn put<T: Canon>(_t: &T) -> Id {
        panic!("No store feature selected")
    }

    pub(crate) fn fetch(_id: &Id, _into: &mut [u8]) -> Result<(), CanonError> {
        Err(CanonError::NotFound)
    }

    pub(crate) fn id<T: Canon>(_t: &T) -> Id {
        panic!("No store feature selected")
    }

    pub(crate) fn hash(_bytes: &[u8]) -> [u8; 32] {
        panic!("No store feature selected")
    }
}
