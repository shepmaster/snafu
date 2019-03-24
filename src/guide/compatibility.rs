//! ## Rust version compatibility
//!
//! SNAFU is tested and compatible back to Rust 1.18, released on
//! 2017-06-08. Compatibility is controlled by Cargo feature flags.
//!
//! ### Default
//!
//! - Targets the current stable version of Rust at the time of
//!   release of the crate. Check the Cargo.toml for the exact
//!   version.
//!
//! ### No features - supports Rust 1.18
//!
//! - Implements [`Error`](std::error::Error) and [`Display`](std::fmt::Display).
//! - Creates context selectors.
//!
//! ### `rust_1_30` - supports Rust 1.30
//!
//! - Adds an implementation for [`Error::source`](std::error::Error::source).
//! - Adds support for re-exporting the `Snafu` macro directly from
//!   the `snafu` crate.
