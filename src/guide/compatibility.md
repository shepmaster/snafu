## Rust version compatibility

SNAFU is tested and compatible back to Rust 1.34, released on
2019-05-14. Compatibility is controlled by Cargo feature flags.

## `rust_1_46`

**default**: enabled

When enabled, SNAFU will assume that it's safe to target features
available in Rust 1.46. Notably, the `#[track_caller]` feature is
needed to allow [`Location`][crate::Location] to automatically discern
the source code location.
