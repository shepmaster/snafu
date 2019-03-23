//! # Attributes understood by the `Snafu` macro
//!
//! ## Controlling `Display`
//!
//! For backwards compatibility purposes, there are a number of ways
//! you can specify how the `Display` trait will be implemented for
//! each variant:
//!
//! - `#[snafu_display("a format string with arguments: {}", "info")]`
//!
//!   Every argument is quoted as a string literal separately.
//!
//! - `#[snafu_display = r#"("a format string with arguments: {}", info)"#]`
//!
//!   The entire
//!
//! Each choice has the same capabilities. All of the fields of the
//! variant will be available and you can call methods on them, such
//! as `filename.display()`.
//!
//! ## Controlling visibility
//!
//! By default, each of the context selectors and their inherent
//! methods will be private. It is our opinion that each module should
//! have one or more error types that are scoped to that module,
//! reducing the need to deal with unrelated errors when matching and
//! increasing cohesiveness.
//!
//! If you need access the context selectors from outside of their
//! module, you can use the `#[snafu_visibility]` attribute. This can
//! be applied to the error type as a default visibility or to
//! specific context selectors.
//!
//! There are two forms of the attribute:
//!
//! - `#[snafu_visibility = "X"]`, where `X` is a normal Rust
//!   visibility modifier (`pub`, `pub(crate)`, `pub(in some::path)`,
//!   etc.)
//! - `#[snafu_visibility]` will reset back to private visibility.
//!
//! ```
//! # use snafu::Snafu;
//! #[derive(Debug, Snafu)]
//! #[snafu_visibility = "pub(crate)"] // Default
//! enum Error {
//!     IsPubCrate, // Uses the default
//!     #[snafu_visibility]
//!     IsPrivate,  // Will be private
//! }
//! ```
//!
//! ## Controlling backtraces
//!
//! If your error contains other SNAFU errors which can report
//! backtraces, you may wish to delegate returning a backtrace to
//! those errors. Use `#[snafu_backtrace(delegate)]` to specify this:
//!
//! ```rust
//! # mod another {
//! #     use snafu::Snafu;
//! #     #[derive(Debug, Snafu)]
//! #     pub enum Error {}
//! # }
//! # use snafu::Snafu;
//! #[derive(Debug, Snafu)]
//! enum Error {
//!     MyError {
//!         #[snafu_backtrace(delegate)]
//!         source: another::Error,
//!     }
//! }
//! ```
