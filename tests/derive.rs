use canon_derive::Canon;
use canonical::{Canon, Store};

mod toy_store;
use toy_store::ToyStore;

#[derive(Canon, PartialEq, Debug)]
struct A {
    a: u64,
    b: u64,
}

#[derive(Canon, PartialEq, Debug)]
struct A2 {
    a: u8,
    b: u8,
}

#[derive(Canon, PartialEq, Debug)]
struct B(u64, u64);

#[derive(Canon, PartialEq, Debug)]
struct C(u64);

#[derive(Canon, PartialEq, Debug)]
struct D;

#[derive(Canon, PartialEq, Debug)]
enum E {
    A,
    B,
}

#[derive(Canon, PartialEq, Debug)]
enum F {
    A(u64, u64),
    B(u8),
}

#[derive(Canon, PartialEq, Debug)]
enum G {
    A { alice: u64, bob: u8 },
    B(u64),
    C,
}

#[derive(Canon, PartialEq, Debug)]
struct H<T>(T);

#[derive(Canon, PartialEq, Debug)]
struct MonsterStruct<T> {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H<T>,
}

fn serialize_deserialize<T: Canon + std::fmt::Debug + PartialEq>(mut t: T) {
    let mut store = ToyStore::new();

    println!("encoding {:?}", &t);

    let id = store.put(&mut t).unwrap();

    println!("store {:?}", &store);

    let restored = store.get::<T>(&id).unwrap().unwrap();

    assert_eq!(t, restored);
}

#[test]
fn derives() {
    serialize_deserialize(A { a: 37, b: 77 });
    serialize_deserialize(A2 { a: 37, b: 77 });
    serialize_deserialize(B(37, 22));
    serialize_deserialize(C(22));
    serialize_deserialize(D);
    serialize_deserialize(E::A);
    serialize_deserialize(E::B);
    serialize_deserialize(F::A(73, 3));
    serialize_deserialize(F::B(22));
    serialize_deserialize(G::A { alice: 73, bob: 3 });
    serialize_deserialize(G::B(73));
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
        f: F::A(73, 3),
        g: G::A { alice: 73, bob: 3 },
        h: H(E::B),
    });
}
