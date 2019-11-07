//! Additions to the [`TryFuture`] trait.
//!
//! [`TryFuture`]: futures_core_crate::future::TryFuture

use crate::{Error, ErrorCompat, IntoError};
use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use futures_core_crate::future::TryFuture;
use pin_project::pin_project;

/// Additions to [`TryFuture`].
pub trait TryFutureExt: TryFuture + Sized {
    /// Extend a [`TryFuture`]'s error with additional context-sensitive
    /// information.
    ///
    /// ```rust
    /// # use futures_crate as futures;
    /// use futures::future::TryFuture;
    /// use snafu::{futures::TryFutureExt, Snafu};
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
    /// fn example() -> impl TryFuture<Ok = i32, Error = Error> {
    ///     another_function().context(Authenticating {
    ///         user_name: "admin",
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn another_function() -> impl TryFuture<Ok = i32, Error = ApiError> {
    ///     /* ... */
    /// # futures::future::ok(42)
    /// }
    /// ```
    ///
    /// Note that the context selector will call [`Into::into`] on
    /// each field, so the types are not required to exactly match.
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::Error>,
        E: Error + ErrorCompat;

    /// Extend a [`TryFuture`]'s error with lazily-generated context-sensitive
    /// information.
    ///
    /// ```rust
    /// # use futures_crate as futures;
    /// use futures::future::TryFuture;
    /// use snafu::{futures::TryFutureExt, Snafu};
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
    /// fn example() -> impl TryFuture<Ok = i32, Error = Error> {
    ///     another_function().with_context(|| Authenticating {
    ///         user_name: "admin".to_string(),
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn another_function() -> impl TryFuture<Ok = i32, Error = ApiError> {
    ///     /* ... */
    /// # futures::future::ok(42)
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

impl<Fut> TryFutureExt for Fut
where
    Fut: TryFuture,
{
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::Error>,
        E: Error + ErrorCompat,
    {
        Context {
            inner: self,
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
            inner: self,
            context: Some(context),
            _e: PhantomData,
        }
    }
}

/// Future for the [`context`](TryFutureExt::context) combinator.
///
/// See the [`TryFutureExt::context`] method for more details.
#[pin_project]
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct Context<Fut, C, E> {
    #[pin]
    inner: Fut,
    context: Option<C>,
    _e: PhantomData<E>,
}

impl<Fut, C, E> Future for Context<Fut, C, E>
where
    Fut: TryFuture,
    C: IntoError<E, Source = Fut::Error>,
    E: Error + ErrorCompat,
{
    type Output = Result<Fut::Ok, E>;

    fn poll(self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Self::Output> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        inner.try_poll(ctx).map_err(|error| {
            context
                .take()
                .expect("Cannot poll Context after it resolves")
                .into_error(error)
        })
    }
}

/// Future for the [`with_context`](TryFutureExt::with_context) combinator.
///
/// See the [`TryFutureExt::with_context`] method for more details.
#[pin_project]
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct WithContext<Fut, F, E> {
    #[pin]
    inner: Fut,
    context: Option<F>,
    _e: PhantomData<E>,
}

impl<Fut, F, C, E> Future for WithContext<Fut, F, E>
where
    Fut: TryFuture,
    F: FnOnce() -> C,
    C: IntoError<E, Source = Fut::Error>,
    E: Error + ErrorCompat,
{
    type Output = Result<Fut::Ok, E>;

    fn poll(self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Self::Output> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        inner.try_poll(ctx).map_err(|error| {
            let context = context
                .take()
                .expect("Cannot poll WithContext after it resolves");

            context().into_error(error)
        })
    }
}
