//! Additions to the Futures 0.1 [`Future`] trait.
//!
//! [`Future`]: futures_01_crate::Future

use crate::{Error, ErrorCompat, IntoError};
use core::marker::PhantomData;
use futures_01_crate::{Async, Future};

/// Additions to [`Future`].
pub trait FutureExt: Future + Sized {
    /// Extend a [`Future`]'s error with additional context-sensitive
    /// information.
    ///
    /// [`Future`]: futures_01_crate::Future]
    ///
    /// ```rust
    /// # use futures_01_crate as futures;
    /// use futures::Future;
    /// # use futures::future;
    /// use snafu::{futures01::FutureExt, Snafu};
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
    /// fn example() -> impl Future<Item = i32, Error = Error> {
    ///     another_function().context(Authenticating {
    ///         user_name: "admin",
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn another_function() -> impl Future<Item = i32, Error = ApiError> {
    ///     /* ... */
    /// # future::ok(42)
    /// }
    /// ```
    ///
    /// Note that the context selector will call [`Into::into`] on
    /// each field, so the types are not required to exactly match.
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::Error>,
        E: Error + ErrorCompat;

    /// Extend a [`Future`]'s error with lazily-generated context-sensitive
    /// information.
    ///
    /// [`Future`]: futures_01_crate::Future]
    ///
    /// ```rust
    /// # use futures_01_crate as futures;
    /// use futures::Future;
    /// # use futures::future;
    /// use snafu::{futures01::FutureExt, Snafu};
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
    /// fn example() -> impl Future<Item = i32, Error = Error> {
    ///     another_function().with_context(|| Authenticating {
    ///         user_name: "admin".to_string(),
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = std::io::Error;
    /// fn another_function() -> impl Future<Item = i32, Error = ApiError> {
    ///     /* ... */
    /// # future::ok(42)
    /// }
    /// ```
    ///
    /// Note that this *may not* be needed in many cases because the
    /// context selector will call [`Into::into`] on each field.
    fn with_context<F, C, E>(self, context: F) -> WithContext<Self, F, E>
    where
        F: FnOnce() -> C,
        C: IntoError<E, Source = Self::Error>,
        E: Error + ErrorCompat;
}

impl<Fut> FutureExt for Fut
where
    Fut: Future,
{
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::Error>,
        E: Error + ErrorCompat,
    {
        Context {
            future: self,
            context: Some(context),
            _e: PhantomData,
        }
    }

    fn with_context<F, C, E>(self, context: F) -> WithContext<Self, F, E>
    where
        F: FnOnce() -> C,
        C: IntoError<E, Source = Self::Error>,
        E: Error + ErrorCompat,
    {
        WithContext {
            future: self,
            context: Some(context),
            _e: PhantomData,
        }
    }
}

/// Future for the [`context`](FutureExt::context) combinator.
///
/// See the [`FutureExt::context`] method for more details.
pub struct Context<Fut, C, E> {
    future: Fut,
    context: Option<C>,
    _e: PhantomData<E>,
}

impl<Fut, C, E> Future for Context<Fut, C, E>
where
    Fut: Future,
    C: IntoError<E, Source = Fut::Error>,
    E: Error + ErrorCompat,
{
    type Item = Fut::Item;
    type Error = E;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.future.poll().map_err(|error| {
            self.context
                .take()
                .expect("cannot poll Context after it has resolved")
                .into_error(error)
        })
    }
}

/// Future for the [`with_context`](FutureExt::with_context) combinator.
///
/// See the [`FutureExt::with_context`] method for more details.
pub struct WithContext<Fut, F, E> {
    future: Fut,
    context: Option<F>,
    _e: PhantomData<E>,
}

impl<Fut, F, C, E> Future for WithContext<Fut, F, E>
where
    Fut: Future,
    F: FnOnce() -> C,
    C: IntoError<E, Source = Fut::Error>,
    E: Error + ErrorCompat,
{
    type Item = Fut::Item;
    type Error = E;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.future.poll().map_err(|error| {
            let context = self
                .context
                .take()
                .expect("cannot poll WithContext after it has resolved");

            context().into_error(error)
        })
    }
}
