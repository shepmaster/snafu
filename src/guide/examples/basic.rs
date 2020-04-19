//! The most common usage of SNAFU — an enumeration of possible errors.
//!
//! Start by looking at the error type [`Error`], then view the
//! *context selectors* [`Leaf`] and [`Intermediate`].

use crate::{Snafu, ResultExt};

/// An enumeration of possible errors.
///
/// This will create a number of *context selectors*:
///
/// - [`Leaf`]
/// - [`Intermediate`]
///
/// ## Leaf errors
///
/// Context selectors for error variants without a `source`, such
/// as `Leaf`, have methods to construct them, such as
/// [`Leaf::build`] or [`Leaf::fail`]. The [`ensure`] macro also
/// accepts these kinds of context selectors.
///
/// ```
/// # use snafu::guide::examples::basic::*;
/// use snafu::ensure;
///
/// fn always_fails() -> Result<(), Error> {
///     Leaf { user_id: 42 }.fail()
/// }
///
/// fn sometimes_fails(user_id: i32) -> Result<(), Error> {
///     ensure!(user_id > 0, Leaf { user_id });
///     Ok(())
/// }
/// ```
///
/// ## Intermediate errors
///
/// Context selectors for error variants with a `source`, such as
/// `Intermediate`, are intended to be used with the
/// [`ResultExt::context`] family of methods.
///
/// ```
/// # use snafu::guide::examples::basic::*;
/// use snafu::ResultExt;
///
/// fn load_config_file() -> Result<usize, Error> {
///     let config = std::fs::read_to_string("/path/to/my/config/file").context(Intermediate)?;
///     Ok(config.len())
/// }
/// ```
#[derive(Debug, Snafu)]
// This line is only needed to generate documentation; it is not
// needed in most cases:
#[snafu(crate_root(crate), visibility = "pub")]
pub enum Error {
    Leaf {
        user_id: i32,
    },

    Intermediate {
        source: std::io::Error,
    },
}
