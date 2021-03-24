// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

/// This file is a workaround to allow for use of the types in Canonical
/// without having to either use the BridgeStore (with associated extern
/// linking points) or the std.
use crate::{CanonError, IdHash};

/// The singleton responsible for saving and restoring values
pub struct VoidStore;

impl VoidStore {
    pub(crate) fn get(
        _hash: &IdHash,
        _into: &mut [u8],
    ) -> Result<(), CanonError> {
        panic!("No store feature selected")
    }

    pub(crate) fn put(_bytes: &[u8]) -> IdHash {
        panic!("No store feature selected")
    }

    pub(crate) fn hash(_bytes: &[u8]) -> IdHash {
        panic!("No store feature selected")
    }
}
