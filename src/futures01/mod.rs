//! Additions to the Futures 0.1 [`Future`] and [`Stream`] traits.
//!
//! This module is only available when the `futures-01` [feature
//! flag] is enabled.
//!
//! [`Future`]: futures_01_crate::Future
//! [`Stream`]: futures_01_crate::Stream
//! [feature flag]: crate::guide::feature_flags

pub mod future;
pub mod stream;

#[doc(inline)]
pub use self::future::FutureExt;
#[doc(inline)]
pub use self::stream::StreamExt;
