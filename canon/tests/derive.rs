// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use canonical::{Canon, Handle};
use canonical_derive::Canon;
use canonical_host::MemStore;

#[derive(Clone, Canon, PartialEq, Debug)]
struct A {
    a: u64,
    b: u64,
}

#[derive(Clone, Canon, PartialEq, Debug)]
struct A2 {
    a: (),
    b: u8,
}

#[derive(Clone, Canon, PartialEq, Debug)]
struct B(u64, u64);

#[derive(Clone, Canon, PartialEq, Debug)]
struct C(u64);

#[derive(Clone, Canon, PartialEq, Debug)]
struct D;

#[derive(Clone, Canon, PartialEq, Debug)]
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

#[derive(Clone, Canon, PartialEq, Debug)]
enum G {
    A { alice: u64, bob: u8 },
    B(Option<u32>),
    C,
}

#[derive(Clone, Canon, PartialEq, Debug)]
struct H<T>(T);

#[derive(Clone, Canon, PartialEq, Debug)]
struct I<T>(Vec<T>);

#[derive(Clone, Canon, PartialEq, Debug)]
struct MonsterStruct<T> {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H<T>,
    i: I<T>,
}

fn serialize_deserialize<
    T: Canon<MemStore> + Clone + std::fmt::Debug + PartialEq,
>(
    t: T,
) {
    let store = MemStore::new();
    let mut handle = Handle::new(t.clone());

    handle.commit(&store).unwrap();

    let restored = handle.restore().unwrap();
    assert_eq!(t, restored);
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
        f: F::A(73, [0, 1, 4, 3, 4]),
        g: G::A { alice: 73, bob: 3 },
        h: H(E::B),
        i: I(vec![E::B, E::B, E::A]),
    });
}
