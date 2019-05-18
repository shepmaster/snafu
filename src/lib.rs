#![deny(missing_docs)]

//! # SNAFU
//!
//! SNAFU is a library to easily assign underlying errors into
//! domain-specific errors while adding context. For detailed
//! information, please see the [user's guide](guide).
//!
//! ## Quick example
//!
//! This example mimics a (very poor) authentication process that
//! opens a file, writes to a file, and checks the user's ID. While
//! two of our operations involve an [`io::Error`](std::io::Error),
//! these are different conceptual errors to us.
//!
//! SNAFU creates *context selectors* mirroring each error
//! variant. These are used with the [`context`](ResultExt::context)
//! method to provide ergonomic error handling.
//!
//! ```rust
//! use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
//! use std::{
//!     fs,
//!     path::{Path, PathBuf},
//! };
//!
//! #[derive(Debug, Snafu)]
//! enum Error {
//!     #[snafu(display("Could not open config from {}: {}", filename.display(), source))]
//!     OpenConfig {
//!         filename: PathBuf,
//!         source: std::io::Error,
//!     },
//!     #[snafu(display("Could not save config to {}: {}", filename.display(), source))]
//!     SaveConfig {
//!         filename: PathBuf,
//!         source: std::io::Error,
//!     },
//!     #[snafu(display("The user id {} is invalid", user_id))]
//!     UserIdInvalid { user_id: i32, backtrace: Backtrace },
//! }
//!
//! type Result<T, E = Error> = std::result::Result<T, E>;
//!
//! fn log_in_user<P>(config_root: P, user_id: i32) -> Result<bool>
//! where
//!     P: AsRef<Path>,
//! {
//!     let config_root = config_root.as_ref();
//!     let filename = &config_root.join("config.toml");
//!
//!     let config = fs::read(filename).context(OpenConfig { filename })?;
//!     // Perform updates to config
//!     fs::write(filename, config).context(SaveConfig { filename })?;
//!
//!     ensure!(user_id == 42, UserIdInvalid { user_id });
//!
//!     Ok(true)
//! }
//!
//! # const CONFIG_DIRECTORY: &str = "/does/not/exist";
//! # const USER_ID: i32 = 0;
//! fn log_in() {
//!     match log_in_user(CONFIG_DIRECTORY, USER_ID) {
//!         Ok(true) => println!("Logged in!"),
//!         Ok(false) => println!("Not logged in!"),
//!         Err(e) => {
//!             eprintln!("An error occurred: {}", e);
//!             if let Some(backtrace) = ErrorCompat::backtrace(&e) {
//!                 println!("{}", backtrace);
//!             }
//!         }
//!     }
//! }
//! ```

#[cfg(feature = "backtraces")]
extern crate backtrace;
#[cfg(feature = "backtraces")]
mod backtrace_shim;
#[cfg(feature = "backtraces")]
pub use backtrace_shim::*;

#[cfg(feature = "rust_1_30")]
extern crate snafu_derive;
#[cfg(feature = "rust_1_30")]
pub use snafu_derive::Snafu;

pub mod guide;

use std::error;

/// Ensure a condition is true. If it is not, return from the function
/// with an error.
///
/// ```rust
/// use snafu::{ensure, Snafu};
///
/// #[derive(Debug, Snafu)]
/// enum Error {
///     InvalidUser { user_id: i32 },
/// }
///
/// fn example(user_id: i32) -> Result<(), Error> {
///     ensure!(user_id > 0, InvalidUser { user_id });
///     // After this point, we know that `user_id` is positive.
///     let user_id = user_id as u32;
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! ensure {
    ($predicate:expr, $context_selector:expr) => {
        if !$predicate {
            return $context_selector.fail();
        }
    };
}

/// Additions to [`Result`](std::result::Result).
pub trait ResultExt<T, E>: Sized {
    /// Extend a [`Result`]'s error with additional context-sensitive information.
    ///
    /// [`Result`]: std::result::Result
    ///
    /// ```rust
    /// use snafu::{ResultExt, Snafu};
    ///
    /// #[derive(Debug, Snafu)]
    /// enum Error {
    ///     Authenticating {
    ///         user_name: String,
    ///         user_id: i32,
    ///         source: ApiError,
    ///     },
    /// }
    ///
    /// fn example() -> Result<(), Error> {
    ///     another_function().context(Authenticating {
    ///         user_name: "admin",
    ///         user_id: 42,
    ///     })?;
    ///     Ok(())
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn another_function() -> Result<i32, ApiError> {
    ///     /* ... */
    /// # Ok(42)
    /// }
    /// ```
    ///
    /// Note that the context selector will call
    /// [`Into::into`](std::convert::Into::into) on each field, so the types
    /// are not required to exactly match.
    fn context<C, E2>(self, context: C) -> Result<T, E2>
    where
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat;

    /// Extend a [`Result`][]'s error with lazily-generated context-sensitive information.
    ///
    /// [`Result`]: std::result::Result
    ///
    /// ```rust
    /// use snafu::{ResultExt, Snafu};
    ///
    /// #[derive(Debug, Snafu)]
    /// enum Error {
    ///     Authenticating {
    ///         user_name: String,
    ///         user_id: i32,
    ///         source: ApiError,
    ///     },
    /// }
    ///
    /// fn example() -> Result<(), Error> {
    ///     another_function().with_context(|| Authenticating {
    ///         user_name: "admin".to_string(),
    ///         user_id: 42,
    ///     })?;
    ///     Ok(())
    /// }
    ///
    /// # type ApiError = std::io::Error;
    /// fn another_function() -> Result<i32, ApiError> {
    ///     /* ... */
    /// # Ok(42)
    /// }
    /// ```
    ///
    /// Note that this *may not* be needed in many cases because the context
    /// selector will call [`Into::into`](std::convert::Into::into) on each
    /// field.
    fn with_context<F, C, E2>(self, context: F) -> Result<T, E2>
    where
        F: FnOnce() -> C,
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat;

    #[doc(hidden)]
    #[deprecated(since = "0.4.0", note = "use ResultExt::context instead")]
    fn eager_context<C, E2>(self, context: C) -> Result<T, E2>
    where
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat,
    {
        self.context(context)
    }

    #[doc(hidden)]
    #[deprecated(since = "0.4.0", note = "use ResultExt::with_context instead")]
    fn with_eager_context<F, C, E2>(self, context: F) -> Result<T, E2>
    where
        F: FnOnce() -> C,
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat,
    {
        self.with_context(context)
    }
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E> {
    fn context<C, E2>(self, context: C) -> Result<T, E2>
    where
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat,
    {
        self.map_err(|error| context.into_error(error))
    }

    fn with_context<F, C, E2>(self, context: F) -> Result<T, E2>
    where
        F: FnOnce() -> C,
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat,
    {
        self.map_err(|error| {
            let context = context();
            context.into_error(error)
        })
    }
}

/// A temporary error type used when converting an [`Option`][] into a
/// [`Result`][]
///
/// [`Option`]: std::option::Option
/// [`Result`]: std::result::Result
pub struct NoneError;

/// Additions to [`Option`](std::option::Option).
pub trait OptionExt<T>: Sized {
    /// Convert an [`Option`][] into a [`Result`][] with additional
    /// context-sensitive information.
    ///
    /// [Option]: std::option::Option
    /// [Result]: std::option::Result
    ///
    /// ```rust
    /// use snafu::{OptionExt, Snafu};
    ///
    /// #[derive(Debug, Snafu)]
    /// enum Error {
    ///     UserLookup { user_id: i32 },
    /// }
    ///
    /// fn example(user_id: i32) -> Result<(), Error> {
    ///     let name = username(user_id).context(UserLookup { user_id })?;
    ///     println!("Username was {}", name);
    ///     Ok(())
    /// }
    ///
    /// fn username(user_id: i32) -> Option<String> {
    ///     /* ... */
    /// # None
    /// }
    /// ```
    ///
    /// Note that the context selector will call
    /// [`Into::into`](std::convert::Into::into) on each field, so the types
    /// are not required to exactly match.
    fn context<C, E>(self, context: C) -> Result<T, E>
    where
        C: IntoError<E, Source = NoneError>,
        E: std::error::Error + ErrorCompat;

    /// Convert an [`Option`][] into a [`Result`][] with
    /// lazily-generated context-sensitive information.
    ///
    /// [`Option`]: std::option::Option
    /// [`Result`]: std::result::Result
    ///
    /// ```
    /// use snafu::{OptionExt, Snafu};
    ///
    /// #[derive(Debug, Snafu)]
    /// enum Error {
    ///     UserLookup {
    ///         user_id: i32,
    ///         previous_ids: Vec<i32>,
    ///     },
    /// }
    ///
    /// fn example(user_id: i32) -> Result<(), Error> {
    ///     let name = username(user_id).with_context(|| UserLookup {
    ///         user_id,
    ///         previous_ids: Vec::new(),
    ///     })?;
    ///     println!("Username was {}", name);
    ///     Ok(())
    /// }
    ///
    /// fn username(user_id: i32) -> Option<String> {
    ///     /* ... */
    /// # None
    /// }
    /// ```
    ///
    /// Note that this *may not* be needed in many cases because the context
    /// selector will call [`Into::into`](std::convert::Into::into) on each
    /// field.
    fn with_context<F, C, E>(self, context: F) -> Result<T, E>
    where
        F: FnOnce() -> C,
        C: IntoError<E, Source = NoneError>,
        E: std::error::Error + ErrorCompat;

    #[doc(hidden)]
    #[deprecated(since = "0.4.0", note = "use OptionExt::context instead")]
    fn eager_context<C, E>(self, context: C) -> Result<T, E>
    where
        C: IntoError<E, Source = NoneError>,
        E: std::error::Error + ErrorCompat,
    {
        self.context(context).map_err(Into::into)
    }

    #[doc(hidden)]
    #[deprecated(since = "0.4.0", note = "use OptionExt::with_context instead")]
    fn with_eager_context<F, C, E>(self, context: F) -> Result<T, E>
    where
        F: FnOnce() -> C,
        C: IntoError<E, Source = NoneError>,
        E: std::error::Error + ErrorCompat,
    {
        self.with_context(context).map_err(Into::into)
    }
}

impl<T> OptionExt<T> for Option<T> {
    fn context<C, E>(self, context: C) -> Result<T, E>
    where
        C: IntoError<E, Source = NoneError>,
        E: std::error::Error + ErrorCompat,
    {
        self.ok_or_else(|| context.into_error(NoneError))
    }

    fn with_context<F, C, E>(self, context: F) -> Result<T, E>
    where
        F: FnOnce() -> C,
        C: IntoError<E, Source = NoneError>,
        E: std::error::Error + ErrorCompat,
    {
        self.ok_or_else(|| context().into_error(NoneError))
    }
}

/// Backports changes to the [`Error`](std::error::Error) trait to
/// versions of Rust lacking them.
///
/// It is recommended to always call these methods explicitly so that
/// it is easy to replace usages of this trait when you start
/// supporting a newer version of Rust.
///
/// ```
/// # use snafu::{Snafu, ErrorCompat};
/// # #[derive(Debug, Snafu)] enum Example {};
/// # fn example(error: Example) {
/// ErrorCompat::backtrace(&error); // Recommended
/// error.backtrace();              // Discouraged
/// # }
/// ```
pub trait ErrorCompat {
    /// Returns a [`Backtrace`](Backtrace) that may be printed.
    #[cfg(feature = "backtraces")]
    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }
}

impl<'a, E> ErrorCompat for &'a E
where
    E: ErrorCompat,
{
    #[cfg(feature = "backtraces")]
    fn backtrace(&self) -> Option<&Backtrace> {
        (**self).backtrace()
    }
}

impl<E> ErrorCompat for Box<E>
where
    E: ErrorCompat,
{
    #[cfg(feature = "backtraces")]
    fn backtrace(&self) -> Option<&Backtrace> {
        (**self).backtrace()
    }
}

/// Converts the receiver into an [`Error`][] trait object, suitable
/// for use in [`Error::source`][].
///
/// It is expected that most users of SNAFU will not directly interact
/// with this trait.
///
/// [`Error`]: std::error::Error
/// [`Error::source`]: std::error::Error::source
//
// Given an error enum with multiple types of underlying causes:
//
// ```rust
// enum Error {
//     BoxTraitObjectSendSync(Box<dyn error::Error + Send + Sync + 'static>),
//     BoxTraitObject(Box<dyn error::Error + 'static>),
//     Boxed(Box<io::Error>),
//     Unboxed(io::Error),
// }
// ```
//
// This trait provides the answer to what consistent expression can go
// in each match arm:
//
// ```rust
// impl error::Error for Error {
//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         use Error::*;
//
//         let v = match *self {
//             BoxTraitObjectSendSync(ref e) => ...,
//             BoxTraitObject(ref e) => ...,
//             Boxed(ref e) => ...,
//             Unboxed(ref e) => ...,
//         };
//
//         Some(v)
//     }
// }
//
// Existing methods like returning `e`, `&**e`, `Borrow::borrow(e)`,
// `Deref::deref(e)`, and `AsRef::as_ref(e)` do not work for various
// reasons.
pub trait AsErrorSource {
    /// For maximum effectiveness, this needs to be called as a method
    /// to benefit from Rust's automatic dereferencing of method
    /// receivers.
    fn as_error_source(&self) -> &(error::Error + 'static);
}

impl AsErrorSource for error::Error + 'static {
    fn as_error_source(&self) -> &(error::Error + 'static) {
        self
    }
}

impl AsErrorSource for error::Error + Send + 'static {
    fn as_error_source(&self) -> &(error::Error + 'static) {
        self
    }
}

impl AsErrorSource for error::Error + Sync + 'static {
    fn as_error_source(&self) -> &(error::Error + 'static) {
        self
    }
}

impl AsErrorSource for error::Error + Send + Sync + 'static {
    fn as_error_source(&self) -> &(error::Error + 'static) {
        self
    }
}

impl<T: error::Error + 'static> AsErrorSource for T {
    fn as_error_source(&self) -> &(error::Error + 'static) {
        self
    }
}

/// Combines an underlying error with additional information
/// about the error.
///
/// It is expected that most users of SNAFU will not directly interact
/// with this trait.
pub trait IntoError<Error>
where
    Error: std::error::Error + ErrorCompat,
{
    /// The underlying error
    type Source;

    /// Combine the information to produce the error
    fn into_error(self, source: Self::Source) -> Error;
}
