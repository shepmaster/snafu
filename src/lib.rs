#![deny(missing_docs)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable-backtraces-impl-std", feature(backtrace))]

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
//! SNAFU creates a *context selector* type for each variant in the
//! error enum. These context selectors are used with the
//! [`context`](ResultExt::context) method to provide ergonomic error
//! handling.
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
//! # #[cfg(not(feature = "backtraces-impl-backtrace-crate"))]
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

#[cfg(all(
    not(feature = "backtraces"),
    not(feature = "backtraces-impl-backtrace-crate"),
    not(feature = "unstable-backtraces-impl-std"),
))]
mod backtrace_inert;
#[cfg(all(
    not(feature = "backtraces"),
    not(feature = "backtraces-impl-backtrace-crate"),
    not(feature = "unstable-backtraces-impl-std"),
))]
pub use crate::backtrace_inert::*;

#[cfg(all(
    feature = "backtraces",
    not(feature = "backtraces-impl-backtrace-crate"),
    not(feature = "unstable-backtraces-impl-std"),
))]
mod backtrace_shim;
#[cfg(all(
    feature = "backtraces",
    not(feature = "backtraces-impl-backtrace-crate"),
    not(feature = "unstable-backtraces-impl-std"),
))]
pub use crate::backtrace_shim::*;

#[cfg(feature = "backtraces-impl-backtrace-crate")]
pub use backtrace::Backtrace;

#[cfg(feature = "unstable-backtraces-impl-std")]
pub use std::backtrace::Backtrace;

#[cfg(feature = "futures-01")]
pub mod futures01;

#[cfg(feature = "futures")]
pub mod futures;

pub use snafu_derive::Snafu;

#[cfg(feature = "guide")]
macro_rules! generate_guide {
    (pub mod $name:ident; $($rest:tt)*) => {
        generate_guide!(@gen ".", pub mod $name { } $($rest)*);
    };
    (pub mod $name:ident { $($children:tt)* } $($rest:tt)*) => {
        generate_guide!(@gen ".", pub mod $name { $($children)* } $($rest)*);
    };
    (@gen $prefix:expr, ) => {};
    (@gen $prefix:expr, pub mod $name:ident; $($rest:tt)*) => {
        generate_guide!(@gen $prefix, pub mod $name { } $($rest)*);
    };
    (@gen $prefix:expr, @code pub mod $name:ident; $($rest:tt)*) => {
        pub mod $name;
        generate_guide!(@gen $prefix, $($rest)*);
    };
    (@gen $prefix:expr, pub mod $name:ident { $($children:tt)* } $($rest:tt)*) => {
        doc_comment::doc_comment! {
            include_str!(concat!($prefix, "/", stringify!($name), ".md")),
            pub mod $name {
                generate_guide!(@gen concat!($prefix, "/", stringify!($name)), $($children)*);
            }
        }
        generate_guide!(@gen $prefix, $($rest)*);
    };
}

#[cfg(feature = "guide")]
generate_guide! {
    pub mod guide {
        pub mod attributes;
        pub mod comparison {
            pub mod failure;
        }
        pub mod compatibility;
        pub mod feature_flags;
        pub mod generics;
        pub mod opaque;
        pub mod philosophy;
        pub mod the_macro;
        pub mod upgrading;

        @code pub mod examples;
    }
}

doc_comment::doctest!("../README.md", readme_tests);

#[cfg(any(feature = "std", test))]
#[doc(hidden)]
pub use std::error::Error;

#[cfg(not(any(feature = "std", test)))]
mod no_std_error;
#[cfg(not(any(feature = "std", test)))]
#[doc(hidden)]
pub use no_std_error::Error;

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
            return $context_selector.fail().map_err(core::convert::Into::into);
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
        E2: Error + ErrorCompat;

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
        E2: Error + ErrorCompat;

    #[doc(hidden)]
    #[deprecated(since = "0.4.0", note = "use ResultExt::context instead")]
    fn eager_context<C, E2>(self, context: C) -> Result<T, E2>
    where
        C: IntoError<E2, Source = E>,
        E2: Error + ErrorCompat,
    {
        self.context(context)
    }

    #[doc(hidden)]
    #[deprecated(since = "0.4.0", note = "use ResultExt::with_context instead")]
    fn with_eager_context<F, C, E2>(self, context: F) -> Result<T, E2>
    where
        F: FnOnce() -> C,
        C: IntoError<E2, Source = E>,
        E2: Error + ErrorCompat,
    {
        self.with_context(context)
    }
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn context<C, E2>(self, context: C) -> Result<T, E2>
    where
        C: IntoError<E2, Source = E>,
        E2: Error + ErrorCompat,
    {
        self.map_err(|error| context.into_error(error))
    }

    fn with_context<F, C, E2>(self, context: F) -> Result<T, E2>
    where
        F: FnOnce() -> C,
        C: IntoError<E2, Source = E>,
        E2: Error + ErrorCompat,
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
        E: Error + ErrorCompat;

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
        E: Error + ErrorCompat;

    #[doc(hidden)]
    #[deprecated(since = "0.4.0", note = "use OptionExt::context instead")]
    fn eager_context<C, E>(self, context: C) -> Result<T, E>
    where
        C: IntoError<E, Source = NoneError>,
        E: Error + ErrorCompat,
    {
        self.context(context).map_err(Into::into)
    }

    #[doc(hidden)]
    #[deprecated(since = "0.4.0", note = "use OptionExt::with_context instead")]
    fn with_eager_context<F, C, E>(self, context: F) -> Result<T, E>
    where
        F: FnOnce() -> C,
        C: IntoError<E, Source = NoneError>,
        E: Error + ErrorCompat,
    {
        self.with_context(context).map_err(Into::into)
    }
}

impl<T> OptionExt<T> for Option<T> {
    fn context<C, E>(self, context: C) -> Result<T, E>
    where
        C: IntoError<E, Source = NoneError>,
        E: Error + ErrorCompat,
    {
        self.ok_or_else(|| context.into_error(NoneError))
    }

    fn with_context<F, C, E>(self, context: F) -> Result<T, E>
    where
        F: FnOnce() -> C,
        C: IntoError<E, Source = NoneError>,
        E: Error + ErrorCompat,
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
    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }
}

impl<'a, E> ErrorCompat for &'a E
where
    E: ErrorCompat,
{
    fn backtrace(&self) -> Option<&Backtrace> {
        (**self).backtrace()
    }
}

#[cfg(any(feature = "std", test))]
impl<E> ErrorCompat for Box<E>
where
    E: ErrorCompat,
{
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
    fn as_error_source(&self) -> &(dyn Error + 'static);
}

impl AsErrorSource for dyn Error + 'static {
    fn as_error_source(&self) -> &(dyn Error + 'static) {
        self
    }
}

impl AsErrorSource for dyn Error + Send + 'static {
    fn as_error_source(&self) -> &(dyn Error + 'static) {
        self
    }
}

impl AsErrorSource for dyn Error + Sync + 'static {
    fn as_error_source(&self) -> &(dyn Error + 'static) {
        self
    }
}

impl AsErrorSource for dyn Error + Send + Sync + 'static {
    fn as_error_source(&self) -> &(dyn Error + 'static) {
        self
    }
}

impl<T: Error + 'static> AsErrorSource for T {
    fn as_error_source(&self) -> &(dyn Error + 'static) {
        self
    }
}

/// Combines an underlying error with additional information
/// about the error.
///
/// It is expected that most users of SNAFU will not directly interact
/// with this trait.
pub trait IntoError<E>
where
    E: Error + ErrorCompat,
{
    /// The underlying error
    type Source;

    /// Combine the information to produce the error
    fn into_error(self, source: Self::Source) -> E;
}

/// Construct a backtrace, allowing it to be optional.
pub trait GenerateBacktrace {
    /// Generate a new backtrace instance
    fn generate() -> Self;

    /// Retrieve the optional backtrace
    fn as_backtrace(&self) -> Option<&Backtrace>;
}

/// Only create a backtrace when an environment variable is set.
///
/// This looks first for the value of `RUST_LIB_BACKTRACE` then
/// `RUST_BACKTRACE`. If the value is set to `1`, backtraces will be
/// enabled.
///
/// This value will be tested only once per program execution;
/// changing the environment variable after it has been checked will
/// have no effect.
#[cfg(any(feature = "std", test))]
impl GenerateBacktrace for Option<Backtrace> {
    fn generate() -> Self {
        use std::env;
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Once,
        };

        static START: Once = Once::new();
        static ENABLED: AtomicBool = AtomicBool::new(false);

        START.call_once(|| {
            // TODO: What values count as "true"?
            let enabled = env::var_os("RUST_LIB_BACKTRACE")
                .or_else(|| env::var_os("RUST_BACKTRACE"))
                .map_or(false, |v| v == "1");
            ENABLED.store(enabled, Ordering::SeqCst);
        });

        if ENABLED.load(Ordering::SeqCst) {
            Some(Backtrace::generate())
        } else {
            None
        }
    }

    fn as_backtrace(&self) -> Option<&Backtrace> {
        self.as_ref()
    }
}

#[cfg(feature = "backtraces-impl-backtrace-crate")]
impl GenerateBacktrace for Backtrace {
    fn generate() -> Self {
        Backtrace::new()
    }

    fn as_backtrace(&self) -> Option<&Backtrace> {
        Some(self)
    }
}

#[cfg(feature = "unstable-backtraces-impl-std")]
impl GenerateBacktrace for Backtrace {
    fn generate() -> Self {
        Backtrace::force_capture()
    }

    fn as_backtrace(&self) -> Option<&Backtrace> {
        Some(self)
    }
}
