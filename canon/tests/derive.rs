// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(min_const_generics)]

use arbitrary::Arbitrary;
use canonical::{Canon, Store};
use canonical_derive::Canon;
use canonical_fuzz::{fuzz_canon, fuzz_canon_iterations};
use canonical_host::MemStore;

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
struct A {
    a: u64,
    b: u64,
}

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
struct A2 {
    a: (),
    b: u8,
}

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
struct B(u64, u64);

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
struct C(u64);

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
struct D;

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
enum E {
    A,
    B,
}

#[derive(Clone, Canon, PartialEq, Debug)]
enum F {
    A(u64, [u64; 5]),
    B(u8),
    C(Result<u32, u32>),
}

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
enum G {
    A { alice: u64, bob: u8 },
    B(Option<u32>),
    C,
}

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
struct H<T>(T);

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
struct I<T>(Vec<T>);

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
struct J(String);

#[derive(Clone, Canon, PartialEq, Debug, Arbitrary)]
struct MonsterStruct<T> {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    g: G,
    h: H<T>,
    i: I<T>,
    j: J,
}

#[derive(Clone, Canon, Debug, Arbitrary)]
struct StoreIncludedA<S: Store> {
    junk: u32,
    store: S,
}

#[derive(Clone, Canon, Debug, Arbitrary)]
struct StoreIncludedB<S>
where
    S: Store,
{
    junk: u32,
    store: S,
}

#[derive(Clone, Canon, Debug, Arbitrary, PartialEq)]
struct ConstGenerics<const N: isize> {
    test: u32,
}

//////////////

fn serialize_deserialize<
    T: Canon<MemStore> + Clone + std::fmt::Debug + PartialEq,
>(
    t: T,
) {
    let store = MemStore::new();

    let id = store.put(&t).unwrap();

    let restored = store.get(&id).unwrap();
    assert_eq!(t, restored);
}

#[test]
fn store_included() {
    let store = MemStore::new();

    let thing_a = StoreIncludedA {
        junk: 32,
        store: store.clone(),
    };

    let id_a = store.put(&thing_a).unwrap();

    let thing_b = StoreIncludedB {
        junk: 32,
        store: store.clone(),
    };

    let id_b = store.put(&thing_b).unwrap();

    assert_eq!(id_a, id_b);
}

#[test]
fn derives() {
    serialize_deserialize(A { a: 37, b: 77 });
    serialize_deserialize(A2 { a: (), b: 77 });
    serialize_deserialize(B(37, 22));
    serialize_deserialize(C(22));
    serialize_deserialize(D);
    serialize_deserialize(E::A);
    serialize_deserialize(E::B);
    serialize_deserialize(F::A(73, [0, 1, 2, 3, 4]));
    serialize_deserialize(F::B(22));
    serialize_deserialize(F::C(Ok(3213)));
    serialize_deserialize(F::C(Err(3213)));
    serialize_deserialize(G::A { alice: 73, bob: 3 });
    serialize_deserialize(G::B(Some(73)));
    serialize_deserialize(G::B(None));
    serialize_deserialize(H(73u8));
    serialize_deserialize(H(73u64));
    serialize_deserialize(H(E::B));
    serialize_deserialize(H(F::B(83)));

    serialize_deserialize(MonsterStruct {
        a: A { a: 37, b: 77 },
        b: B(37, 22),
        c: C(22),
        d: D,
        e: E::A,
        g: G::A { alice: 73, bob: 3 },
        h: H(E::B),
        i: I(vec![E::B, E::B, E::A]),
        j: J("Happy happy joy joy!".into()),
    });
}

#[test]
fn const_generics() {
    let a: ConstGenerics<-1> = ConstGenerics { test: 3 };
    serialize_deserialize(a);
}

#[test]
fn fuzzing() {
    let store = MemStore::new();

    fuzz_canon::<A, _>(store.clone());
    fuzz_canon_iterations::<MonsterStruct<Option<u32>>, _>(128, store);
}
