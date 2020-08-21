# canonical

Canonical is a serialization library built for no_std environments where you want to deal with recursive datastructures, such as trees.

Its main component is the `Canon` trait, which specifies that a type can be written into bytes, and also that the length of the written value is known beforehand.

This greatly simplifies dealing with environments lacking allocations, and provides a convenient way to pass values across FFI-barriers.

# canonical_derive

In order not to have to write all this byte-counting code by hand, canonical includes a derive-macro to implement them for you.

```rust
#[derive(Canon, PartialEq, Debug)]
struct A2 {
    a: u8,
    b: u8,
}
```

For a more involved example, this is a stack structure from the tests.

```rust
#[derive(Canon)]
enum Stack<T, S>
where
    S: Store,
{
    Empty,
    Node { value: T, prev: Handle<Self, S> },
}
```

The `Handle` type here acts as a `Box`, but is supported in non-allocating code, the trick being that the allocation happens outside, in a special `Store` abstraction.

```rust
pub trait Store {
    type Ident: Ident;
    type Error: From<CanonError>;

    fn put<T: Canon>(t: &mut T) -> Result<Self::Ident, Self::Error>;
    fn get<T: Canon>(id: &Self::Ident) -> Result<T, Self::Error>;
}
```

The `Ident` is a value used to refer to encoded values, this is generally a hash of some sort of the encoded bytes.

In a wasm no_std environment, the `put` and `get` of `Store` can be implemented as a host call, and effectively do the allocations "host-side".