# Canonical, the serialization and wasm abi framework

It's split into 4 main crates

# Crate structure

## canon

The actual trait that is used to serialize/deserialize values to/from bytes.

The trait also provides a way to know how many bytes a value will consume once written, allowing efficient handling of resources and allocations in the host.

## canon_derive

The automatic derivation of `Canon` for structs, and enums.

```rust
use canonical_derive::Canon;

#[derive(Clone, Canon)]
pub struct Test {
    hello: i32,
		world: u32,
}
```

## canon_host

The plumbing for the host responsible for the wasm host calls. Also contains the MemoryStore ephemeral backend for testing purposes.

## remote_derive

Derivation macros for constructing "remotes", which are logic that is keeping track of it's state and provides. The usual case of a "remote" is a contract with a certain state running queries or transactions.

# General idea

To keep remotes/contracts as simple and small (in bytecode) as possible, the ABI and the serialization format is unified into one. the `Canon` trait. Additionally, the need for an allocator is removed, as values can be stored in the host environment, referenced by their Hashes or similar host-specified identification methods.

This allows for the remote contracts to define their own data layouts and storage features.

# ABI communication

The host communicates with the remotes by queries and transactions.

A query is a call that does not modify any of the contracts state comparable to a `&self` method in rust. Likewise, a transaction corresponds to a `&mut self` method.

The layout of the call (from the host side) is

```
tag: u8 - which query/transaction is being called
data: [
	[u8; *] - self representation in bytes
	[u8; *] - argument representation in bytes
]
```

Since the read out value of `self` in the remote method call has to implement `Canon`, it can calculate where the argument data starts, which removes the need to specify lengths in the ABI data protocol.
