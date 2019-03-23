//! ## Optional extensions to the crate
//!
//! In addition to the feature flags [controlling compatibility],
//! there are Cargo [feature flags] that extend SNAFU for various use
//! cases.
//!
//! [controlling compatibility]: super::guide::compatibility
//! [feature flags]: https://doc.rust-lang.org/stable/cargo/reference/specifying-dependencies.html#choosing-features
//!
//! ### `backtraces`
//!
//! **default**: enabled
//!
//! When enabled, you can use the [`Backtrace`](crate::Backtrace) type in
//! your enum variant. If you never use backtraces, you can omit this
//! feature to speed up compilation a small amount.
