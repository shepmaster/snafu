use alloc::{boxed::Box, string::String};

use crate::{Backtrace, ChainCompat, Snafu};

/// A basic error type that you can use as a first step to better
/// error handling.
///
/// You can use this type in your own application as a quick way to
/// create errors or add basic context to another error. This can also
/// be used in a library, but consider wrapping it in an
/// [opaque](crate::guide::opaque) error to avoid putting the SNAFU
/// crate in your public API.
///
/// ## Examples
///
/// ```rust
/// use snafu::prelude::*;
///
/// type Result<T, E = snafu::Whatever> = std::result::Result<T, E>;
///
/// fn subtract_numbers(a: u32, b: u32) -> Result<u32> {
///     if a > b {
///         Ok(a - b)
///     } else {
///         whatever!("Can't subtract {a} - {b}")
///     }
/// }
///
/// fn complicated_math(a: u32, b: u32) -> Result<u32> {
///     let val = subtract_numbers(a, b).whatever_context("Can't do the math")?;
///     Ok(val * 2)
/// }
/// ```
///
/// See [`whatever!`][crate::whatever!] for detailed usage instructions.
///
/// ## Limitations
///
/// When wrapping errors, only the backtrace from the shallowest
/// function is guaranteed to be available. If you need the deepest
/// possible trace, consider creating a custom error type and [using
/// `#[snafu(backtrace)]` on the `source`
/// field](Snafu#controlling-backtraces). If a best-effort attempt is
/// sufficient, see the [`backtrace`][Self::backtrace] method.
///
/// When the standard library stabilizes support for the
/// [provide API](https://doc.rust-lang.org/std/error/trait.Error.html#method.provide),
/// this behavior may change.
///
/// ## Thread Safety
///
/// This type requires that contained errors implement [`Send`][] and
/// [`Sync`][]. If this is burdensome, you may also use
/// [`WhateverLocal`][].
#[derive(Debug, Snafu)]
#[snafu(crate_root(crate))]
#[snafu(whatever)]
#[snafu(display("{message}"))]
#[snafu(provide(opt, ref, chain, dyn crate::Error + Send + Sync => source.as_deref()))]
pub struct Whatever {
    #[snafu(source(from(Box<dyn crate::Error + Send + Sync>, Some)))]
    #[snafu(provide(false))]
    source: Option<Box<dyn crate::Error + Send + Sync>>,
    message: String,
    backtrace: Backtrace,
}

impl Whatever {
    /// Gets the backtrace from the deepest [`Whatever`][] or
    /// [`WhateverLocal`][] error. If none of the underlying errors
    /// are one of these types, returns the backtrace from when this
    /// instance was created.
    pub fn backtrace(&self) -> &Backtrace {
        known_whatevers_backtrace(self).unwrap_or(&self.backtrace)
    }
}

/// A basic error type that you can use as a first step to better
/// error handling when the error does not need to cross a thread
/// boundary.
///
/// This type behaves the same as [`Whatever`][] except it does not
/// require that the wrapped errors implement [`Send`][] or
/// [`Sync`][]. See [`Whatever`][] and [`whatever!`][crate::whatever!] for detailed
/// usage instructions.
#[derive(Debug, Snafu)]
#[snafu(crate_root(crate))]
#[snafu(whatever)]
#[snafu(display("{message}"))]
#[snafu(provide(opt, ref, chain, dyn crate::Error => source.as_deref()))]
pub struct WhateverLocal {
    #[snafu(source(from(Box<dyn crate::Error>, Some)))]
    #[snafu(provide(false))]
    source: Option<Box<dyn crate::Error>>,
    message: String,
    backtrace: Backtrace,
}

impl WhateverLocal {
    /// Gets the backtrace from the deepest [`Whatever`][] or
    /// [`WhateverLocal`][] error. If none of the underlying errors
    /// are one of these types, returns the backtrace from when this
    /// instance was created.
    pub fn backtrace(&self) -> &Backtrace {
        known_whatevers_backtrace(self).unwrap_or(&self.backtrace)
    }
}

fn known_whatevers_backtrace<'a>(
    root: &'a (dyn crate::Error + 'static),
) -> Option<&'a crate::Backtrace> {
    ChainCompat::new(root)
        .skip(1)
        .filter_map(|e| {
            if let Some(e) = e.downcast_ref::<Whatever>() {
                Some(&e.backtrace)
            } else if let Some(e) = e.downcast_ref::<WhateverLocal>() {
                Some(&e.backtrace)
            } else {
                None
            }
        })
        .last()
}
