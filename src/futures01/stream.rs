//! Additions to the Futures 0.1 [`Stream`] trait.
//!
//! [`Stream`]: futures_01_crate::Stream

use crate::{Error, ErrorCompat, IntoError};
use core::marker::PhantomData;
use futures_01_crate::{Async, Stream};

/// Additions to [`Stream`].
pub trait StreamExt: Stream + Sized {
    /// Extend a [`Stream`]'s error with additional context-sensitive
    /// information.
    ///
    /// [`Stream`]: futures_01_crate::Stream]
    ///
    /// ```rust
    /// # use futures_01_crate as futures;
    /// use futures::Stream;
    /// # use futures::stream;
    /// use snafu::{futures01::StreamExt, Snafu};
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
    /// fn example() -> impl Stream<Item = i32, Error = Error> {
    ///     stock_prices().context(Authenticating {
    ///         user_name: "admin",
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn stock_prices() -> impl Stream<Item = i32, Error = ApiError> {
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

    /// Extend a [`Stream`]'s error with lazily-generated context-sensitive
    /// information.
    ///
    /// [`Stream`]: futures_01_crate::Stream]
    ///
    /// ```rust
    /// # use futures_01_crate as futures;
    /// use futures::Stream;
    /// # use futures::stream;
    /// use snafu::{futures01::StreamExt, Snafu};
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
    /// fn example() -> impl Stream<Item = i32, Error = Error> {
    ///     stock_prices().with_context(|| Authenticating {
    ///         user_name: "admin".to_string(),
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = std::io::Error;
    /// fn stock_prices() -> impl Stream<Item = i32, Error = ApiError> {
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

impl<St> StreamExt for St
where
    St: Stream,
{
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::Error> + Clone,
        E: Error + ErrorCompat,
    {
        Context {
            stream: self,
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
            stream: self,
            context,
            _e: PhantomData,
        }
    }
}

/// Stream for the [`context`](StreamExt::context) combinator.
///
/// See the [`StreamExt::context`] method for more details.
pub struct Context<St, C, E> {
    stream: St,
    context: C,
    _e: PhantomData<E>,
}

impl<St, C, E> Stream for Context<St, C, E>
where
    St: Stream,
    C: IntoError<E, Source = St::Error> + Clone,
    E: Error + ErrorCompat,
{
    type Item = St::Item;
    type Error = E;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.stream
            .poll()
            .map_err(|error| self.context.clone().into_error(error))
    }
}

/// Stream for the [`with_context`](StreamExt::with_context) combinator.
///
/// See the [`StreamExt::with_context`] method for more details.
pub struct WithContext<St, F, E> {
    stream: St,
    context: F,
    _e: PhantomData<E>,
}

impl<St, F, C, E> Stream for WithContext<St, F, E>
where
    St: Stream,
    F: FnMut() -> C,
    C: IntoError<E, Source = St::Error>,
    E: Error + ErrorCompat,
{
    type Item = St::Item;
    type Error = E;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.stream.poll().map_err(|error| {
            let context = &mut self.context;
            context().into_error(error)
        })
    }
}
