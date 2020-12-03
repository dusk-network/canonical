// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use arbitrary::Arbitrary;
use canonical::{Canon, Repr, Store};
use canonical_derive::Canon;
use canonical_fuzz::fuzz_canon;
use canonical_host::MemStore;

// We don't want PartialEq on the Repr for performance reasons, so in the test
// we use a newtype
#[derive(Clone, Canon, Debug)]
struct ReprWrap<T, S: Store>(Repr<T, S>);

impl<T, S> Arbitrary for ReprWrap<T, S>
where
    T: 'static + Canon<S> + Arbitrary,
    S: Store,
{
    fn arbitrary(
        u: &mut arbitrary::Unstructured<'_>,
    ) -> arbitrary::Result<Self> {
        Ok(ReprWrap(Repr::arbitrary(u)?))
    }
}

impl<T, S> PartialEq for ReprWrap<T, S>
where
    T: Canon<S>,
    S: Store,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.get_id() == other.0.get_id()
    }
}

#[test]
fn fuzz_repr() {
    let store = MemStore::new();
    fuzz_canon::<ReprWrap<u32, MemStore>, MemStore>(store);
}
