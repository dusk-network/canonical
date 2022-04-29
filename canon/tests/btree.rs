// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::collections::{BTreeMap, BTreeSet};

use canonical::{Canon, Sink, Source};

#[test]
fn test_btree_map() {
    let mut map: BTreeMap<i32, i32> = BTreeMap::new();

    map.insert(1, 2);
    map.insert(3, 4);
    map.insert(5, 6);

    let mut buf = vec![0; map.encoded_len()];
    let mut sink = Sink::new(&mut buf);

    map.encode(&mut sink);
    drop(sink);

    let mut source = Source::new(&buf);

    assert_eq!(map, BTreeMap::decode(&mut source).unwrap());
}

#[test]
fn test_btree_set() {
    let mut set: BTreeSet<i32> = BTreeSet::new();

    set.insert(1);
    set.insert(2);
    set.insert(3);

    let mut buf = vec![0; set.encoded_len()];
    let mut sink = Sink::new(&mut buf);

    set.encode(&mut sink);
    drop(sink);

    let mut source = Source::new(&buf);

    assert_eq!(set, BTreeSet::decode(&mut source).unwrap());
}
