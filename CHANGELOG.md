# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.4] - 2019-08-07

### Fixed

- Ignore `#[doc]` attributes that do not correspond to documentation
  comments. This allows `#[doc(hidden)]` to be used again.

### Changed

- Implement `Future` and `Stream` instead of `TryFuture` and
  `TryStream` for the combinators for the standard library's
  futures. This allows the `Context` future combinator to be directly
  used with `.await` and for the `Context` stream combinator to be
  used without calling `.into_stream`.

[0.4.4]: https://github.com/shepmaster/snafu/releases/tag/0.4.4

## [0.4.3] - 2019-07-23

### Added

- Add optional conversion of `&snafu::Backtrace` into `&backtrace::Backtrace`.

### Fixed

- Support default generic parameters on error types.

[0.4.3]: https://github.com/shepmaster/snafu/releases/tag/0.4.3

## [0.4.2] - 2019-07-21

### Added

- Documentation comment summaries are used as the default `Display` text.

### Fixed

- Quieted warnings from usages of bare trait objects.
- The `From` trait is fully-qualified to avoid name clashes.

### Changed

- More errors are reported per compilation attempt.

[0.4.2]: https://github.com/shepmaster/snafu/releases/tag/0.4.2

## [0.4.1] - 2018-05-18

### Fixed

- A feature flag name was rejected by crates.io and needed to be
  updated; this release has no substantial changes beyond 0.4.0.

[0.4.1]: https://github.com/shepmaster/snafu/releases/tag/0.4.1

## [0.4.0] - 2018-05-18

### Added

- Context selectors now automatically implement `Debug`, `Copy`, and
  `Clone`. This is a **breaking change**.

- Support for futures 0.1 futures and streams is available using the
  `futures-01` feature flag.

- **Experimental** support for standard library futures and streams is
  available using the `unstable-futures` feature flag.

### Deprecated

- `eager_context` and `with_eager_context` have been deprecated.

### Removed

- The `Context` type is no longer needed. This is a **breaking
  change**.

- SNAFU types no longer implement `Borrow<std::error::Error>`. This is
  a **breaking change**.

[0.4.0]: https://github.com/shepmaster/snafu/releases/tag/0.4.0

## [0.3.1] - 2019-05-10

### Fixed

- Underlying error causes of `Box<dyn std::error::Error + Send +
  Sync>` are now supported.

### Deprecated

- `Borrow` is no longer required to be implemented for underlying
  error causes. In the next release containing breaking changes, the
  automatic implementation of `Borrow<dyn std::error::Error>` for
  SNAFU types will be removed.

[0.3.1]: https://github.com/shepmaster/snafu/releases/tag/0.3.1

## [0.3.0] - 2019-05-08

### Added

- `Borrow<std::error::Error>` is now automatically implemented for
  SNAFU types. This is a **breaking change** as it may conflict with
  an existing user implementation of the same trait. It is expected
  that the number of affected users is very small.

- `#[snafu(source)]` can be used to identify the field that
  corresponds to the underlying error if it is not called `source`. It
  can also be used to disable automatically using a field called
  `source` for the underlying error.

- `#[snafu(backtrace)]` can be used to identify the field that
  corresponds to the backtrace if it is not called `backtrace`. It can
  also be used to disable automatically using a field called
  `backtrace` for the backtrace.

- `#[snafu(source(from(...type..., ...expression...)))]` can be used
  to perform transformations on the underlying error before it is
  stored. This allows boxing of large errors to avoid bloated return
  types or recursive errors.

- The user guide has a basic comparison to Failure and migration paths
  for common Failure patterns.

### Changed

- The default `Display` implementation includes the underlying error
  message.

[0.3.0]: https://github.com/shepmaster/snafu/releases/tag/0.3.0

## [0.2.3] - 2019-04-24

### Fixed

- User-provided `where` clauses on error types are now copied to
  SNAFU-created `impl` blocks.
- User-provided inline trait bounds (`<T: SomeTrait>`) are no longer
  included in SNAFU-generated type names.

[0.2.3]: https://github.com/shepmaster/snafu/releases/tag/0.2.3

## [0.2.2] - 2019-04-19

### Fixed

- Error enums with variants named `Some` or `None` no longer cause
  name conflicts in the generated code.

[0.2.2]: https://github.com/shepmaster/snafu/releases/tag/0.2.2

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
