//! Additions to the [`TryStream`] trait.
//!
//! [`TryStream`]: futures_core_crate::TryStream

use crate::{Error, ErrorCompat, IntoError};
use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use futures_core_crate::stream::{Stream, TryStream};
use pin_project::pin_project;

/// Additions to [`TryStream`].
pub trait TryStreamExt: TryStream + Sized {
    /// Extend a [`TryStream`]'s error with additional context-sensitive
    /// information.
    ///
    /// ```rust
    /// # use futures_crate as futures;
    /// use futures::TryStream;
    /// # use futures::stream;
    /// use snafu::{futures::TryStreamExt, Snafu};
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
    /// fn example() -> impl TryStream<Ok = i32, Error = Error> {
    ///     stock_prices().context(Authenticating {
    ///         user_name: "admin",
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn stock_prices() -> impl TryStream<Ok = i32, Error = ApiError> {
    ///     /* ... */
    /// # stream::empty()
    /// }
    /// ```
    ///
    /// Note that the context selector will call [`Into::into`] on
    /// each field, so the types are not required to exactly match.
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::Error> + Clone,
        E: Error + ErrorCompat;

    /// Extend a [`TryStream`]'s error with lazily-generated
    /// context-sensitive information.
    ///
    /// ```rust
    /// # use futures_crate as futures;
    /// use futures::TryStream;
    /// # use futures::stream;
    /// use snafu::{futures::TryStreamExt, Snafu};
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
    /// fn example() -> impl TryStream<Ok = i32, Error = Error> {
    ///     stock_prices().with_context(|| Authenticating {
    ///         user_name: "admin",
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn stock_prices() -> impl TryStream<Ok = i32, Error = ApiError> {
    ///     /* ... */
    /// # stream::empty()
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

impl<St> TryStreamExt for St
where
    St: TryStream,
{
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::Error> + Clone,
        E: Error + ErrorCompat,
    {
        Context {
            inner: self,
            context,
            _e: PhantomData,
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

/// Stream for the [`context`](TryStreamExt::context) combinator.
///
/// See the [`TryStreamExt::context`] method for more details.
#[pin_project]
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Context<St, C, E> {
    #[pin]
    inner: St,
    context: C,
    _e: PhantomData<E>,
}

impl<St, C, E> Stream for Context<St, C, E>
where
    St: TryStream,
    C: IntoError<E, Source = St::Error> + Clone,
    E: Error + ErrorCompat,
{
    type Item = Result<St::Ok, E>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        match inner.try_poll_next(ctx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Ok(v))) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(Some(Err(error))) => {
                let error = context.clone().into_error(error);
                Poll::Ready(Some(Err(error)))
            }
        }
    }
}

/// Stream for the [`with_context`](TryStreamExt::with_context) combinator.
///
/// See the [`TryStreamExt::with_context`] method for more details.
#[pin_project]
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct WithContext<St, F, E> {
    #[pin]
    inner: St,
    context: F,
    _e: PhantomData<E>,
}

impl<St, F, C, E> Stream for WithContext<St, F, E>
where
    St: TryStream,
    F: FnMut() -> C,
    C: IntoError<E, Source = St::Error>,
    E: Error + ErrorCompat,
{
    type Item = Result<St::Ok, E>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let inner = this.inner;
        let context = this.context;

        match inner.try_poll_next(ctx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Ok(v))) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(Some(Err(error))) => {
                let error = context().into_error(error);
                Poll::Ready(Some(Err(error)))
            }
        }
    }
}
