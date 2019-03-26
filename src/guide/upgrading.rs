//! # Upgrading from previous releases
//!
//! ## Version 0.2
//!
//! Support for the `snafu::display` attribute was removed as this
//! type of attribute was [never intended to be
//! supported][oops]. Since this required a SemVer-incompatible
//! version, the attribute format has also been updated and
//! normalized.
//!
//! 1. Attributes have been renamed
//!     - `snafu_display` and `snafu::display` became `snafu(display)`.
//!     - `snafu_visibility` became `snafu(visibility)`
//!     - `snafu_backtrace` became `snafu(backtrace)`
//!
//! 1. Support for `snafu_display` with individually-quoted format
//!    arguments was removed. Migrate to either the "clean" or "all
//!    one string" styles, depending on what version of Rust you are
//!    targeting.
//!
//! [oops]: https://github.com/rust-lang/rust/pull/58899
//!
//! ### Before
//!
//! ```rust,ignore
//! #[derive(Debug, Snafu)]
//! enum DisplayUpdate {
//!     #[snafu::display("Format and {}", argument)]
//!     CleanStyle { argument: i32 },
//!
//!     #[snafu_display("Format and {}", "argument")]
//!     QuotedArgumentStyle { argument: i32 },
//!
//!     #[snafu_display = r#"("Format and {}", argument)"#]
//!     AllOneStringStyle { argument: i32 },
//! }
//! ```
//!
//! ```rust,ignore
//! #[derive(Debug, Snafu)]
//! enum VisibilityUpdate {
//!     #[snafu_visibility(pub(crate))]
//!     CleanStyle,
//!
//!     #[snafu_visibility = "pub(crate)"]
//!     AllOneStringStyle,
//! }
//! ```
//!
//! ### After
//!
//! ```rust,ignore
//! # use snafu::Snafu;
//! #[derive(Debug, Snafu)]
//! enum DisplayUpdate {
//!     #[snafu(display("Format and {}", argument))]
//!     CleanStyle { argument: i32 },
//!
//!     #[snafu(display = r#"("Format and {}", argument)"#)]
//!     QuotedArgumentStyle { argument: i32 },
//!
//!     #[snafu(display = r#"("Format and {}", argument)"#)]
//!     AllOneStringStyle { argument: i32 },
//! }
//! ```
//!
//! ```rust,ignore
//! # use snafu::Snafu;
//! #[derive(Debug, Snafu)]
//! enum VisibilityUpdate {
//!     #[snafu(visibility(pub(crate)))]
//!     CleanStyle,
//!
//!     #[snafu(visibility = "pub(crate)")]
//!     AllOneStringStyle,
//! }
//! ```
//!
