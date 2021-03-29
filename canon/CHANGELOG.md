# Changelog

## [Unreleased]

### Removed

- Remove the `Store` trait

### Changed

- The trait `Canon<S>` becomes `Canon`

- `Canon::<S>::write(&type, &mut Sink<S>)?;` becomes `type.encode(&mut sink);`

- Writes are assumed to always succeed

- The names encode/decode is choosen not to confuse them with `std::io::{Read, Write}`

- Page size limitation removed on encoding/decoding

## [0.5.3] 2021-03-04

### Changed
- Page size to 32k

## [0.5.2] 2021-02-23

### Changed
- Page size increase

### Removed
- Remove #[no_mangle] from extern 'C' functions

## [0.5.1] 2021-02-25

### Added

- Implement `Canon` for `Vec`

## [0.5.0] 2021-01-19

### Changed

- ByteSink/ByteSource no longer needs to clone the store reference

### Removed

- Wasmi integration, moved to dusk-vm where it belongs

## [0.4.4] 2020-12-22

### Added

- Support for arrays of 33 elements.
- Example module using `nstack`.

## [0.4.1] 2020-11-06

### Added 

- Catching of panics and propagating of them to the host

## [0.4.0] 2020-10-21

- Changes in workspace dependencies

## [0.3.0] 2020-10-20

- Changes in workspace dependencies

## [0.2.1] 2020-10-16

- Changes in workspace dependencies

## [0.2.0] 2020-10-15

### Added

- Fuzzing tests for the `Repr` type

- `Canon` implementation for `String`

### Removed

- Remove `hosted` feature

- Const generics

## [0.1.0] 2020-10-06

Initial release
