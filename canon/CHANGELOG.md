# Changelog

## [Unreleased]

## [0.6.5] 2021-06-04

### Added
- Add back public exports of Val and ValMut

## [0.6.4] 2021-06-04

### Changed
- Change `Id::encoded_len` to a correct implementation

### Deprecated

- Deprecate `Repr`

## [0.6.3] 2021-05-26

### Added
- Add varint support for `u128` and `i128` by converting them to two `u64`s
- Add encoding/decoding tests for all integer types

### Changed
- Change library `integer-encoding` to `dusk-varint`

## [0.6.2] 2021-05-20

### Added
- Add method `take_bytes` to `Id`

### Changed

- Change `Store` to be thread local
- Change the payload length in `Id` to u32
- Change encoding of integers to use varints

## [0.6.1] 2021-05-03

### Changed

- Replace macro for arrays with const generics

## [0.6.0] 2021-03-30

### Added

- Add target architecture `wasm32` to conditional compile the `Store` backend

### Changed

- Change trait `Canon<S>` to `Canon`
- Change `Canon::{read,write}` to `Canon::{encode, decode}` not avoid confusion with `std::io::{Read, Write}`

### Removed

- Remove the `Store` trait
- Remove Result on Canon `write`/`encode`, is assumed to always succeed
- Remove page size limitation removed on encoding/decoding
- Remove all features
- Remove legacy `debug` module

### Fixed

- Fix various Clippy hints

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

- Change `ByteSink` / `ByteSource` to no longer require cloning the store reference

### Removed

- Remove wasmi integration, moved to `dusk-abi` where it belongs

## [0.4.4] 2020-12-22

### Added

- Add support for arrays of 33 elements
- Add example module using `nstack`

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

[Unreleased]: https://github.com/dusk-network/canonical/compare/canonical-0.6.5...HEAD
[0.6.5]: https://github.com/dusk-network/canonical/compare/canonical-0.6.4...canonical-0.6.5
[0.6.4]: https://github.com/dusk-network/canonical/compare/canonical-0.6.3...canonical-0.6.4
[0.6.3]: https://github.com/dusk-network/canonical/compare/canonical-0.6.2...canonical-0.6.3
[0.6.2]: https://github.com/dusk-network/canonical/compare/canonical-0.6.1...canonical-0.6.2
[0.6.1]: https://github.com/dusk-network/canonical/compare/canonical-0.6.0...canonical-0.6.1
[0.6.0]: https://github.com/dusk-network/canonical/compare/canonical-0.5.3...canonical-0.6.0
[0.5.3]: https://github.com/dusk-network/canonical/compare/canonical-0.5.2...canonical-0.5.3
[0.5.2]: https://github.com/dusk-network/canonical/compare/canonical-0.5.1...canonical-0.5.2
[0.5.1]: https://github.com/dusk-network/canonical/compare/canonical-0.5.0...canonical-0.5.1
[0.5.0]: https://github.com/dusk-network/canonical/compare/canonical-0.4.4...canonical-0.5.0
[0.4.4]: https://github.com/dusk-network/canonical/compare/canonical-0.4.1...canonical-0.4.4
[0.4.1]: https://github.com/dusk-network/canonical/compare/canonical-0.4.0...canonical-0.4.1
[0.4.0]: https://github.com/dusk-network/canonical/compare/canonical-0.3.0...canonical-0.4.0
[0.3.0]: https://github.com/dusk-network/canonical/compare/canonical-0.2.1...canonical-0.3.0
[0.2.1]: https://github.com/dusk-network/canonical/compare/canonical-0.2.0...canonical-0.2.1
[0.2.0]: https://github.com/dusk-network/canonical/compare/canonical-0.1.0...canonical-0.2.0
[0.1.0]: https://github.com/dusk-network/canonical/releases/tag/canonical-0.1.0
