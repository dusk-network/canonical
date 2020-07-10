use canon::{Canon, InvalidEncoding, Sink, Store};
use canon_derive::Canon;

mod toy_store;
use toy_store::ToyStore;

#[derive(Canon, PartialEq, Debug)]
struct A {
    a: u64,
    b: u64,
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
}

#[derive(Canon, PartialEq, Debug)]
struct H<T>(T);

fn serialize_deserialize<T: Canon + std::fmt::Debug + PartialEq>(t: T) {
    let mut store = ToyStore::new();

    let mut sink = store.sink();
    t.write(&mut sink);

    let id = sink.fin();

    assert_eq!(
        T::read(&mut store.source(&id).expect("missing value"))
            .expect("invalid encoding"),
        t
    );
}

#[test]
fn derives() {
    serialize_deserialize(A { a: 37, b: 77 });
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
}
