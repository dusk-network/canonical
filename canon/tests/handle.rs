// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![allow(clippy::unit_cmp)]

use canonical::{Repr, Store};

#[test]
fn zero_sized_reprs() {
    let repr = Repr::new(());

    let id = Store::put(&repr);

    let restored = Store::get(&id).unwrap();
    assert_eq!((), restored);
}
