# Canonical

![Build Status](https://github.com/dusk-network/canonical/workflows/Continuous%20integration/badge.svg)
[![Repository](https://img.shields.io/badge/github-canonical-blueviolet?logo=github)](https://github.com/dusk-network/canonical)
[![Documentation](https://img.shields.io/badge/docs-canonical-blue?logo=rust)](https://docs.rs/canonical/)

Canonical is a specialized serialization library built for merkle trees and suitable for wasm environments.

Its main component is the `Canon` trait, which specifies how the type is encoded to/read from bytes.

This allows you, for example, to easily pass complex sets containing millions of elements between different wasm modules.

## identifiers

Each value of a type implementing the `Canon` trait has a 1:1 relation to an `Id`.

```rust
let a = 42;

let id = Id::new(&a);

assert_eq!(id.reify().expect("read back"), a);
```

The `Repr<T>` is a smart-pointer type that either owns the value, contains a cryptographic hash of the value, or both. This allows you to construct recursive data types, that can also effeciently be stored and accessed as merkle trees.

# canonical_derive

In order not to have to write all this byte-counting code by hand, canonical includes a derive-macro to implement them for you.

```rust
#[derive(Canon, PartialEq, Debug)]
struct A2 {
    a: u8,
    b: u8,
}
```

# canonical_fuzz

A simple fuzzer built on top of the `arbitrary` crate. Allows you to fuzz the canon encoding for types, helpful if you choose to implement custom encodings.
