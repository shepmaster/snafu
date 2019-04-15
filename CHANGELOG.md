# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2019-04-14

### Added

- Deriving `Snafu` on a newtype struct now creates an opaque error
  type, suitable for conservative public APIs.

[0.2.1]: https://github.com/shepmaster/snafu/releases/tag/0.2.1

## [0.2.0] - 2019-03-02

### Removed

- `snafu::display` and `snafu_display` have been replaced with `snafu(display)`
- `snafu_visibility` has been replaced with `snafu(visibility)`

### Added

- Backtraces can now be delegated to an underlying error via
  `#[snafu(backtrace(delegate))]`.

[0.2.0]: https://github.com/shepmaster/snafu/releases/tag/0.2.0

## [0.1.9] - 2019-03-02

### Added

- Error enums with generic lifetimes and types are now supported.

### Changed

- The trait bounds applied to the `fail` method have been moved from
  the implementation block to the function itself.

[0.1.9]: https://github.com/shepmaster/snafu/releases/tag/0.1.9

## [0.1.8] - 2019-02-27

### Fixed

- Visibility is now applied to context selector fields.

[0.1.8]: https://github.com/shepmaster/snafu/releases/tag/0.1.8

## [0.1.7] - 2019-02-27

### Added

- `#[snafu_visibility]` can be used to configure the visibility of
  context selectors.

[0.1.7]: https://github.com/shepmaster/snafu/releases/tag/0.1.7

## [0.1.6] - 2019-02-24

### Added

- The `OptionExt` extension trait is now available for converting
  `Option`s into `Result`s while adding context.

[0.1.6]: https://github.com/shepmaster/snafu/releases/tag/0.1.6

## [0.1.5] - 2019-02-05

### Changed

- Errors from the macro are more detailed and point to reasonable
  sections of code.

[0.1.5]: https://github.com/shepmaster/snafu/releases/tag/0.1.5

## [0.1.4] - 2019-02-05

### Added

- The `ensure` macro is now available.

[0.1.4]: https://github.com/shepmaster/snafu/releases/tag/0.1.4

## [0.1.3] - 2019-02-04

### Added

- Ability to automatically capture backtraces.

### Changed

- Version requirements for dependencies loosened to allow compiling
  with more crate versions.

[0.1.3]: https://github.com/shepmaster/snafu/releases/tag/0.1.3

## [0.1.2] - 2019-02-02

### Added

- Support for Rust 1.18

[0.1.2]: https://github.com/shepmaster/snafu/releases/tag/0.1.2

## [0.1.1] - 2019-02-01

### Added

- Context selectors without an underlying source now have a `fail`
  method.

- `ResultExt` now has the `eager_context` and `with_eager_context`
   methods to eagerly convert a source `Result` into a final `Result`
   type, skipping the intermediate `Result<_, Context<_>>` type.

[0.1.1]: https://github.com/shepmaster/snafu/releases/tag/0.1.1

## [0.1.0] - 2019-01-27

Initial version

[0.1.0]: https://github.com/shepmaster/snafu/releases/tag/0.1.0
