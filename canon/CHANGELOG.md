# Changelog

## [Unreleased]

### Removed

- Remove the `Store` trait

### Changed

- Change trait `Canon<S>` to `Canon`

- Change the names of `Canon::{read,write}` to `{encode, decode}`  not to confuse them with `std::io::{Read, Write}`

### Removed

- Remove Result on Canon `write`/`encode`, is assumed to always succeed.

- Remove page size limitation removed on encoding/decoding

## [0.5.3] 2021-03-04

### Changed
- Change page size to 32k

## [0.5.2] 2021-02-23

### Changed
- Change page size to be larger

### Removed
- Remove #[no_mangle] from extern 'C' functions

## [0.5.1] 2021-02-25

### Added

- Add an implementation of `Canon` for `Vec`

## [0.5.0] 2021-01-19

### Changed

- Change ByteSink/ByteSource to no longer require cloning the store reference

### Removed

- Remove wasmi integration, moved to dusk-vm where it belongs

## [0.4.4] 2020-12-22

### Added

- Add support for arrays of 33 elements.
- Add example module using `nstack`.

## [0.4.1] 2020-11-06

### Added 

- Add catching of panics and propagating of them to the host

## [0.4.0] 2020-10-21

- Changes in workspace dependencies

## [0.3.0] 2020-10-20

- Changes in workspace dependencies

## [0.2.1] 2020-10-16

- Changes in workspace dependencies

## [0.2.0] 2020-10-15

### Added

- Add fuzzing tests for the `Repr` type

- Add `Canon` implementation for `String`

### Removed

- Remove `hosted` feature

- Remove Const generics

## [0.1.0] 2020-10-06

Initial release
