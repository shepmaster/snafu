# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
