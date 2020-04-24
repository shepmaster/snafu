//! Additions to the [`TryFuture`] and [`TryStream`] traits.
//!
//! This module is only available when the `futures` [feature flag] is
//! enabled.
//!
//! [`TryFuture`]: futures_core_crate::TryFuture
//! [`TryStream`]: futures_core_crate::TryStream
//! [feature flag]: crate::guide::feature_flags

pub mod try_future;
#[cfg(feature = "sink")]
pub mod try_sink;
pub mod try_stream;

#[doc(inline)]
pub use self::try_future::TryFutureExt;
#[cfg(feature = "sink")]
#[doc(inline)]
pub use self::try_sink::SnafuSinkExt;
#[doc(inline)]
pub use self::try_stream::TryStreamExt;
