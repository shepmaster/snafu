## Optional extensions to the crate

In addition to the feature flags [controlling compatibility],
there are Cargo [feature flags] that extend SNAFU for various use
cases.

[controlling compatibility]: super::guide::compatibility
[feature flags]: https://doc.rust-lang.org/stable/cargo/reference/specifying-dependencies.html#choosing-features

### `backtraces`

**default**: disabled

When enabled, the [`Backtrace`] type in your enum variant will capture
a backtrace when the error is generated. If you never use backtraces,
you can omit this feature to speed up compilation a small amount.

It is recommended that only applications make use of this feature.

[`Backtrace`]: crate::Backtrace

### `backtraces-impl-backtrace-crate`

**default**: disabled

When enabled, the SNAFU [`Backtrace`] type becomes an alias to the
`backtrace::Backtrace` type. This allows interoperability with other
crates that require this type.

It is recommended that only applications make use of this feature. At
some point in the future, the standard library will have its own
backtrace type that SNAFU will use and this feature will conflict with
its use.

### `futures`

**default**: disabled

When enabled, you can use the [`futures::TryFutureExt`] and
[`futures::TryStreamExt`] traits to add context methods to futures
and streams returning `Result`s.

Note that this feature requires Rust 1.39.

[`futures::TryFutureExt`]: crate::futures::TryFutureExt
[`futures::TryStreamExt`]: crate::futures::TryStreamExt

### `futures-01`

**default**: disabled

When enabled, you can use the [`futures01::FutureExt`] and
[`futures01::StreamExt`] traits to add context methods to futures
and streams.

[`futures01::FutureExt`]: crate::futures01::FutureExt
[`futures01::StreamExt`]: crate::futures01::StreamExt
