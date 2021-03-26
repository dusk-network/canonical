# Changelog

## [unreleased]

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

[unreleased]: https://github.com/dusk-network/canonical/compare/v0.5.3...HEAD
[0.5.3]: https://github.com/dusk-network/canonical/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/dusk-network/canonical/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/dusk-network/canonical/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/dusk-network/canonical/compare/v0.4.4...v0.5.0
[0.4.4]: https://github.com/dusk-network/canonical/compare/v0.4.1...v0.4.4
[0.4.1]: https://github.com/dusk-network/canonical/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/dusk-network/canonical/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/dusk-network/canonical/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/dusk-network/canonical/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/dusk-network/canonical/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/dusk-network/canonical/releases/tag/v0.1.0
