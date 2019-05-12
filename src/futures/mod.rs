//! Additions to the [`TryFuture`] and [`TryStream`] traits.
//!
//! This module is only available when the `unstable-futures` [feature
//! flag] is enabled.
//!
//! [`TryFuture`]: futures_core::TryFuture
//! [`TryStream`]: futures_core::TryStream
//! [feature flag]: crate::guide::feature_flags

pub mod try_future;
pub mod try_stream;

#[doc(inline)]
pub use self::try_future::TryFutureExt;
#[doc(inline)]
pub use self::try_stream::TryStreamExt;
