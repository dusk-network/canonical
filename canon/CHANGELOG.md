# 0.6.0

## Changed

Deprecate the usage of the `Store` trait.

the trait `Canon<S>` becomes `Canon`.

`Canon::<S>::write(&type, &mut Sink<S>)?;` becomes `type.encode(&mut sink);`

Writes are assumed to always succeed, analgous to how a push to a `Vec` in rust can only panic with out of memory exceptions, but is assumed to be failsafe.

The names encode/decode is choosen not to confuse them with `std::io::{Read, Write}`.

# 0.5.3 a9b53e4e0ebe96fa7bd90ce2ce03875d5e21124a

# Removed

Page size limitation removed on encoding/decoding

## Changes
page size to 32k

# 0.5.2 81eadfa974f17362c223af11fd1d4765a14a1b51

## Changes
page size increase

# 0.5.1 21e85f6ec78e96cd438d078b47b9739d5917759f

REVERTED

# 0.5.0 12062d2c91c4dd48337d8630cb0e200290a4c422

## Changed

ByteSink/ByteSource no longer needs to clone the store reference.

## Added

Implement `Canon` for `Vec`

## Removed

Wasmi integration, moved to dusk-vm where it belongs

# 0.4.4 2b661ddd1249185f45e5731f5699fd50e6a63624

## Added

Support arrays of 33 elements.

# 0.4.1 1d7d6422088709d0363054276a47a20f7b034b01

## Added 

Catching of panics and propagating of them to the host

# 0.4.0 ee0e5104a260793f0d32066c3bc80fcfca70420e

Changes in deprecated repos

# 0.3.0 b25b7b2466eaba2061cacd092ac4eba3eb643aef

Changes in deprecated repos

# 0.2.1 85db137857c3cda61a8ffd2e10b345eeadd40027

Changes in deprecated repos

# 0.2.0 3d331c887a7b6da2d4d044ab38196c14f64d974f

## Added

Fuzzing tests for the `Repr` type

`Canon` implementation for `String`

## Removed

Remove `hosted` feature.

Const generics.

