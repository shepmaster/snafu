//! # SNAFU user's guide
//!
//! Once you've got a high-level idea of what SNAFU can do by looking
//! at the [quick example](crate), take a peek at [our design
//! philosophy](guide::philosophy).
//!
//! For more advanced usage, take a deeper dive into [how the `Snafu`
//! macro works](guide::the_macro), how to create [opaque error
//! types](guide::opaque), and what [attributes are
//! available](guide::attributes).
//!
//! For optional features of the crate, see our [list of feature
//! flags](guide::feature_flags).
//!
//! If you are targeting an older release of Rust, you will be
//! interested in [the compatibility section](guide::compatibility).
//!
//! For upgrading from a previous version, review the [upgrading
//! guide](guide::upgrading).

pub mod attributes;
pub mod compatibility;
pub mod feature_flags;
pub mod opaque;
pub mod philosophy;
pub mod the_macro;
pub mod upgrading;
