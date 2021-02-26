// // This Source Code Form is subject to the terms of the Mozilla Public
// // License, v. 2.0. If a copy of the MPL was not distributed with this
// // file, You can obtain one at http://mozilla.org/MPL/2.0/.
// //
// // Copyright (c) DUSK NETWORK. All rights reserved.

// use arbitrary::Arbitrary;
// use canonical::{Canon, Repr};
// use canonical_derive::Canon;
// use canonical_fuzz::fuzz_canon;

// // We don't want PartialEq on the Repr for performance reasons, so in the
// test // we use a newtype
// #[derive(Clone, Canon, Debug)]
// struct ReprWrap<T>(Repr<T>)
// where
//     T: Canon;

// impl<T> Arbitrary for ReprWrap<T>
// where
//     T: 'static + Canon + Arbitrary,
// {
//     fn arbitrary(
//         u: &mut arbitrary::Unstructured<'_>,
//     ) -> arbitrary::Result<Self> {
//         Ok(ReprWrap(Repr::arbitrary(u)?))
//     }
// }

// impl<T> PartialEq for ReprWrap<T>
// where
//     T: Canon,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.0.get_id() == other.0.get_id()
//     }
// }

// #[test]
// fn fuzz_repr() {
//     fuzz_canon::<ReprWrap<u32>>();
// }
