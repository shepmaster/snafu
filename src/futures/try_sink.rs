//! Additions to the [`TryStream`] trait.
//!
//! [`TryStream`]: futures_core_crate::TryStream

use crate::{Error, ErrorCompat, IntoError, ResultExt};
use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use futures_crate::{Sink, SinkExt};
use pin_project::pin_project;

/// Additions to [`SinkExt`].
pub trait SnafuSinkExt<I>: SinkExt<I> + Sized {
    /// Extend a [`SinkExt`]'s error with additional context-sensitive
    /// information.
    ///
    /// ```rust
    /// # use futures_crate as futures;
    /// use futures::Sink;
    /// # use futures::sink;
    /// use snafu::{futures::SnafuSinkExt, Snafu};
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
    /// fn example() -> impl Sink<i32, Error = Error> {
    ///     transactions().context(Authenticating {
    ///         user_name: "admin",
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn transactions() -> impl Sink<i32, Error = ApiError> {
    ///     /* ... */
    /// # sink::drain()
    /// }
    /// ```
    ///
    /// Note that the context selector will call [`Into::into`] on
    /// each field, so the types are not required to exactly match.
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::Error> + Clone,
        E: Error + ErrorCompat;

    /// Extend a [`SinkExt`]'s error with lazily-generated
    /// context-sensitive information.
    ///
    /// ```rust
    /// # use futures_crate as futures;
    /// use futures::Sink;
    /// # use futures::sink;
    /// use snafu::{futures::SnafuSinkExt, Snafu};
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
    /// fn example() -> impl Sink<i32, Error = Error> {
    ///     transactions().with_context(|| Authenticating {
    ///         user_name: "admin",
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn transactions() -> impl Sink<i32, Error = ApiError> {
    ///     /* ... */
    /// # sink::drain()
    /// }
    /// ```
    ///
    /// Note that this *may not* be needed in many cases because the
    /// context selector will call [`Into::into`] on each field.
    fn with_context<F, C, E>(self, context: F) -> WithContext<Self, F, E>
    where
        F: FnMut() -> C,
        C: IntoError<E, Source = Self::Error>,
        E: Error + ErrorCompat;
}

impl<Si, I> SnafuSinkExt<I> for Si
where
    Si: Sink<I>,
{
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::Error> + Clone,
        E: Error + ErrorCompat,
    {
        Context {
            inner: self,
            context,
            _e: Default::default(),
        }
    }

    fn with_context<F, C, E>(self, context: F) -> WithContext<Self, F, E>
    where
        F: FnMut() -> C,
        C: IntoError<E, Source = Self::Error>,
        E: Error + ErrorCompat,
    {
        WithContext {
            inner: self,
            context,
            _e: PhantomData,
        }
    }
}

/// Sink for the [`context`](SnafuStreamExt::context) combinator.
///
/// See the [`SnafuStreamExt::context`] method for more details.
#[pin_project]
#[derive(Debug)]
#[must_use = "sinks do nothing unless polled"]
pub struct Context<Si, C, E> {
    #[pin]
    inner: Si,
    context: C,
    _e: PhantomData<E>,
}

impl<Sk, C, E, I> Sink<I> for Context<Sk, C, E>
where
    Sk: SinkExt<I>,
    C: IntoError<E, Source = Sk::Error> + Clone,
    E: Error + ErrorCompat,
{
    type Error = E;

    fn poll_ready(self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        inner
            .poll_ready(ctx)
            .map_err(|e| context.clone().into_error(e))
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        inner.start_send(item).context(context.clone())
    }

    fn poll_flush(self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        inner
            .poll_flush(ctx)
            .map_err(|e| context.clone().into_error(e))
    }

    fn poll_close(self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        inner
            .poll_close(ctx)
            .map_err(|e| context.clone().into_error(e))
    }
}

/// Sink for the [`with_context`](SnafuSinkExt::with_context) combinator.
///
/// See the [`SnafuSinkExt::with_context`] method for more details.
#[pin_project]
#[derive(Debug)]
#[must_use = "sinks do nothing unless polled"]
pub struct WithContext<St, F, E> {
    #[pin]
    inner: St,
    context: F,
    _e: PhantomData<E>,
}

impl<Si, F, C, I, E> Sink<I> for WithContext<Si, F, E>
where
    Si: Sink<I>,
    F: FnMut() -> C,
    C: IntoError<E, Source = Si::Error>,
    E: Error + ErrorCompat,
{
    type Error = E;

    fn poll_ready(self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        inner.poll_ready(ctx).map_err(|e| context().into_error(e))
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        inner.start_send(item).with_context(context)
    }

    fn poll_flush(self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        inner.poll_flush(ctx).map_err(|e| context().into_error(e))
    }

    fn poll_close(self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        inner.poll_close(ctx).map_err(|e| context().into_error(e))
    }
}
