## Optional extensions to the crate

In addition to the feature flags [controlling compatibility],
there are Cargo [feature flags] that extend SNAFU for various use
cases.

[controlling compatibility]: super::guide::compatibility
[feature flags]: https://doc.rust-lang.org/stable/cargo/reference/specifying-dependencies.html#choosing-features

### `backtraces`

**default**: enabled

When enabled, you can use the [`Backtrace`](crate::Backtrace) type in
your enum variant. If you never use backtraces, you can omit this
feature to speed up compilation a small amount.

### `unstable-futures`

**default**: disabled

When enabled, you can use the [`futures::TryFutureExt`] and
[`futures::TryStreamExt`] traits to add context methods to futures
and streams returning `Result`s.

Note that this feature requires nightly Rust and may break at any
time. When the standard library implementation stabilizes, this
feature flag will be renamed and stabilized.

[`futures::TryFutureExt`]: crate::futures::TryFutureExt
[`futures::TryStreamExt`]: crate::futures::TryStreamExt

### `futures-01`

**default**: disabled

When enabled, you can use the [`futures01::FutureExt`] and
[`futures01::StreamExt`] traits to add context methods to futures
and streams.

[`futures01::FutureExt`]: crate::futures01::FutureExt
[`futures01::StreamExt`]: crate::futures01::StreamExt
