#![deny(missing_docs)]

//! Crate docs

use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
/// Enum docs
pub enum Error {
    /// Variant docs
    Variant,
}
