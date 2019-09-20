
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.3] - 2019-MM-DD

### Changed

- Adding support for plain Rust comments via special `quote!`-able struct `sourcegen_cli::tokens::PlainComment`.

[0.3.3]: https://github.com/commure/sourcegen/releases/tag/sourcegen-cli-v0.3.3

## [0.3.2] - 2019-08-21

### Changed

- Normalizing doc attributes into `///` without relying on a nightly rustfmt feature.

[0.3.2]: https://github.com/commure/sourcegen/releases/tag/sourcegen-cli-v0.3.2

## [0.3.1] - 2019-08-15

### Changed

- Adding support for nested included modules.

[0.3.1]: https://github.com/commure/sourcegen/releases/tag/sourcegen-cli-v0.3.1

## [0.3.0] - 2019-08-14

### Changed

- Upgraded `proc_macro2` and `syn` dependencies to `1.0`

[0.3.0]: https://github.com/commure/sourcegen/releases/tag/sourcegen-cli-v0.3.0

## [0.2.2] - 2019-08-06

### Changed

- Fixed issue with missing newline on the last line of generated file

[0.2.2]: https://github.com/commure/sourcegen/releases/tag/sourcegen-cli-v0.2.2

## [0.2.1] - 2019-08-06

### Added

- Support for generating full files both via `#![sourcegen::sourcegen(..)]` top-level macro and workaround via `file = true` attribute.

[0.2.1]: https://github.com/commure/sourcegen/releases/tag/sourcegen-cli-v0.2.1

## [0.2.0] - 2019-08-04

### Added

- Support for generated impls via `#[sourcegen::generated]` attribute
- Tests

### Changed

- Fixed issue with newlines on Windows

[0.2.0]: https://github.com/commure/sourcegen/releases/tag/sourcegen-cli-v0.2.0

## [0.1.0] - 2019-07-30

Initial version

[0.1.0]: https://github.com/commure/sourcegen/releases/tag/sourcegen-v0.1.0
