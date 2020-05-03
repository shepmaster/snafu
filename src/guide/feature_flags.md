# Optional extensions to the crate

In addition to the feature flags [controlling compatibility],
there are Cargo [feature flags] that extend SNAFU for various use
cases:

- [`std`](#std)
- [`guide`](#guide)
- [`backtraces`](#backtraces)
- [`backtraces-impl-backtrace-crate`](#backtraces-impl-backtrace-crate)
- [`unstable-backtraces-impl-std`](#unstable-backtraces-impl-std)
- [`futures`](#futures)
- [`futures-01`](#futures-01)

[controlling compatibility]: super::guide::compatibility
[feature flags]: https://doc.rust-lang.org/stable/cargo/reference/specifying-dependencies.html#choosing-features

## `std`

**default**: enabled

When enabled, SNAFU will implement the `std::error::Error` trait. When
disabled, SNAFU will instead implement a custom `Error` trait that is
similar, but does not need any features from the standard library.

Most usages of SNAFU will want this feature enabled.

## `guide`

**default**: enabled

When enabled, the `guide` module containing the user's guide will be
built.

Most usages of SNAFU will want this feature disabled. A future release
will disable this by default.

## `backtraces`

**default**: disabled

When enabled, the [`Backtrace`] type in your enum variant will capture
a backtrace when the error is generated. If you never use backtraces,
you can omit this feature to speed up compilation a small amount.

It is recommended that only applications make use of this feature.

[`Backtrace`]: crate::Backtrace

## `backtraces-impl-backtrace-crate`

**default**: disabled

When enabled, the SNAFU [`Backtrace`] type becomes an alias to the
`backtrace::Backtrace` type. This allows interoperability with other
crates that require this type.

It is recommended that only applications make use of this
feature. When the standard library stabilizes its own backtrace type,
this feature will no longer be supported and will be removed.

## `unstable-backtraces-impl-std`

**default**: disabled

When enabled, the SNAFU [`Backtrace`] type becomes an alias to the
[`std::backtrace::Backtrace`] type and `std::error::Error::backtrace`
is implemented.

It is recommended that only applications make use of this feature.

## `futures`

**default**: disabled

When enabled, you can use the [`futures::TryFutureExt`] and
[`futures::TryStreamExt`] traits to add context methods to futures
and streams returning `Result`s.

[`futures::TryFutureExt`]: crate::futures::TryFutureExt
[`futures::TryStreamExt`]: crate::futures::TryStreamExt

## `futures-01`

**default**: disabled

When enabled, you can use the [`futures01::FutureExt`] and
[`futures01::StreamExt`] traits to add context methods to futures
and streams.

[`futures01::FutureExt`]: crate::futures01::FutureExt
[`futures01::StreamExt`]: crate::futures01::StreamExt
