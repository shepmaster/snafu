//! Additions to the Futures 0.1 [`Sink`] trait.
//!
//! [`Sink`]: futures_01_crate::Sink

use crate::{Error, ErrorCompat, IntoError, ResultExt};
use core::marker::PhantomData;
use futures_01_crate::{Async, AsyncSink, Sink};

/// Additions to [`Sink`].
pub trait SinkExt: Sink + Sized {
    /// Extend a [`Sink`]'s error with additional context-sensitive
    /// information.
    ///
    /// [`Sink`]: futures_01_crate::Sink]
    ///
    /// ```rust
    /// # use futures_01_crate as futures;
    /// use futures::Sink;
    /// use snafu::{futures01::SinkExt, Snafu};
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
    /// fn example() -> impl Sink<SinkItem = i32, SinkError = Error> {
    ///     stock_prices().context(Authenticating {
    ///         user_name: "admin",
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn stock_prices() -> impl Sink<SinkItem = i32, SinkError = ApiError> {
    ///     /* ... */
    /// # Vec::new()
    /// }
    /// ```
    ///
    /// Note that the context selector will call [`Into::into`] on
    /// each field, so the types are not required to exactly match.
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::SinkError> + Clone,
        E: Error + ErrorCompat;

    /// Extend a [`Sink`]'s error with lazily-generated context-sensitive
    /// information.
    ///
    /// [`Sink`]: futures_01_crate::Sink]
    ///
    /// ```rust
    /// # use futures_01_crate as futures;
    /// use futures::Sink;
    /// # use futures::stream;
    /// use snafu::{futures01::SinkExt, Snafu};
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
    /// fn example() -> impl Sink<SinkItem = i32, SinkError = Error> {
    ///     stock_prices().with_context(|| Authenticating {
    ///         user_name: "admin".to_string(),
    ///         user_id: 42,
    ///     })
    /// }
    ///
    /// # type ApiError = std::io::Error;
    /// fn stock_prices() -> impl Sink<SinkItem = i32, SinkError = ApiError> {
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
        C: IntoError<E, Source = Self::SinkError>,
        E: Error + ErrorCompat;
}

impl<St> SinkExt for St
where
    St: Sink,
{
    fn context<C, E>(self, context: C) -> Context<Self, C, E>
    where
        C: IntoError<E, Source = Self::SinkError> + Clone,
        E: Error + ErrorCompat,
    {
        Context {
            sink: self,
            context,
            _e: PhantomData,
        }
    }

    fn with_context<F, C, E>(self, context: F) -> WithContext<Self, F, E>
    where
        F: FnMut() -> C,
        C: IntoError<E, Source = Self::SinkError>,
        E: Error + ErrorCompat,
    {
        WithContext {
            sink: self,
            context,
            _e: PhantomData,
        }
    }
}

/// Sink for the [`context`](SinkExt::context) combinator.
///
/// See the [`SinkExt::context`] method for more details.
pub struct Context<Si, C, E> {
    sink: Si,
    context: C,
    _e: PhantomData<E>,
}

impl<Si, C, E> Sink for Context<Si, C, E>
where
    Si: Sink,
    C: IntoError<E, Source = Si::SinkError> + Clone,
    E: Error + ErrorCompat,
{
    type SinkItem = Si::SinkItem;
    type SinkError = E;

    // fn poll(&mut self) -> Result<Async<Option<Self::SinkItem>>, Self::SinkError> {
    //     self.sink
    //         .poll()
    //         .map_err(|error| self.context.clone().into_error(error))
    // }

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        self.sink.start_send(item).context(self.context.clone())
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sink.poll_complete().context(self.context.clone())
    }

    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sink.close().context(self.context.clone())
    }
}

/// Sink for the [`with_context`](SinkExt::with_context) combinator.
///
/// See the [`SinkExt::with_context`] method for more details.
pub struct WithContext<Si, F, E> {
    sink: Si,
    context: F,
    _e: PhantomData<E>,
}

impl<St, F, C, E> Sink for WithContext<St, F, E>
where
    St: Sink,
    F: FnMut() -> C,
    C: IntoError<E, Source = St::SinkError>,
    E: Error + ErrorCompat,
{
    type SinkItem = St::SinkItem;
    type SinkError = E;

    // fn poll(&mut self) -> Result<Async<Option<Self::SinkItem>>, Self::SinkError> {
    //     self.sink.poll().map_err(|error| {
    //         let context = &mut self.context;
    //         context().into_error(error)
    //     })
    // }

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        self.sink.start_send(item).with_context(|| (self.context)())
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sink.poll_complete().with_context(|| (self.context)())
    }

    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sink.close().with_context(|| (self.context)())
    }
}
