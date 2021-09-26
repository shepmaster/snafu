use futures::{executor::block_on, prelude::*};
use snafu::{location, prelude::*, Location};

#[derive(Debug, Copy, Clone, Snafu)]
struct InnerError {
    location: Location,
}

#[derive(Debug, Copy, Clone, Snafu)]
struct WrappedError {
    source: InnerError,
    location: Location,
}

#[derive(Debug, Snafu)]
struct ManuallyWrappedError {
    source: InnerError,
    #[snafu(implicit(false))]
    location: Location,
}

#[derive(Debug, Snafu)]
#[snafu(display("{}", message))]
#[snafu(whatever)]
pub struct MyWhatever {
    #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
    source: Option<Box<dyn std::error::Error>>,
    message: String,
    location: Location,
}

mod try_future {
    use super::*;

    #[test]
    fn location_macro_uses_creation_location() {
        block_on(async {
            let base_line = line!();
            let error_future = async { InnerSnafu.fail::<()>() };
            let wrapped_error_future = error_future.with_context(|| ManuallyWrappedSnafu {
                location: location!(),
            });
            let wrapped_error = wrapped_error_future.await.unwrap_err();

            assert_eq!(
                wrapped_error.location.line,
                base_line + 3,
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }

    #[test]
    fn async_block_uses_creation_location() {
        block_on(async {
            let base_line = line!();
            let error_future = async { InnerSnafu.fail::<()>() };
            let wrapped_error_future = async { error_future.await.context(WrappedSnafu) };
            let wrapped_error = wrapped_error_future.await.unwrap_err();

            assert_eq!(
                wrapped_error.location.line,
                base_line + 2,
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }

    #[test]
    fn track_caller_is_applied_on_context_poll() {
        block_on(async {
            let base_line = line!();
            let error_future = async { InnerSnafu.fail::<()>() };
            let wrapped_error_future = error_future.context(WrappedSnafu);
            let wrapped_error = wrapped_error_future.await.unwrap_err();

            // `.await` calls our implementation of `poll`, so the
            // location corresponds to that line.
            assert_eq!(
                wrapped_error.location.line,
                base_line + 3,
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }

    #[test]
    fn track_caller_is_applied_on_with_context_poll() {
        block_on(async {
            let base_line = line!();
            let error_future = async { InnerSnafu.fail::<()>() };
            let wrapped_error_future = error_future.with_context(|| WrappedSnafu);
            let wrapped_error = wrapped_error_future.await.unwrap_err();

            // `.await` calls our implementation of `poll`, so the
            // location corresponds to that line.
            assert_eq!(
                wrapped_error.location.line,
                base_line + 3,
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }

    #[test]
    fn track_caller_is_applied_on_whatever_context_poll() {
        block_on(async {
            let base_line = line!();
            let error_future = async { InnerSnafu.fail::<()>() };
            let wrapped_error_future = error_future.whatever_context("bang");
            let wrapped_error: MyWhatever = wrapped_error_future.await.unwrap_err();

            // `.await` calls our implementation of `poll`, so the
            // location corresponds to that line.
            assert_eq!(
                wrapped_error.location.line,
                base_line + 3,
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }

    #[test]
    fn track_caller_is_applied_on_with_whatever_context_poll() {
        block_on(async {
            let base_line = line!();
            let error_future = async { InnerSnafu.fail::<()>() };
            let wrapped_error_future = error_future.with_whatever_context(|_| "bang");
            let wrapped_error: MyWhatever = wrapped_error_future.await.unwrap_err();

            // `.await` calls our implementation of `poll`, so the
            // location corresponds to that line.
            assert_eq!(
                wrapped_error.location.line,
                base_line + 3,
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }
}

mod try_stream {
    use super::*;

    #[test]
    fn location_macro_uses_creation_location() {
        block_on(async {
            let base_line = line!();
            let error_stream = stream::repeat(InnerSnafu.fail::<()>());
            let mut wrapped_error_stream = error_stream.with_context(|| ManuallyWrappedSnafu {
                location: location!(),
            });
            let wrapped_error = wrapped_error_stream.next().await.unwrap().unwrap_err();

            assert_eq!(
                wrapped_error.location.line,
                base_line + 3,
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }

    #[test]
    fn async_block_uses_creation_location() {
        block_on(async {
            let base_line = line!();
            let error_stream = stream::repeat(InnerSnafu.fail::<()>());
            let mut wrapped_error_stream = error_stream.map(|r| r.context(WrappedSnafu));
            let wrapped_error = wrapped_error_stream.next().await.unwrap().unwrap_err();

            assert_eq!(
                wrapped_error.location.line,
                base_line + 2,
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }

    #[test]
    fn track_caller_is_applied_on_context_poll() {
        block_on(async {
            let error_stream = stream::repeat(InnerSnafu.fail::<()>());
            let mut wrapped_error_stream = error_stream.context(WrappedSnafu);
            let wrapped_error = wrapped_error_stream.next().await.unwrap().unwrap_err();

            // `StreamExt::next` doesn't have `[track_caller]`, so the
            // location is inside the futures library.
            assert!(
                wrapped_error.location.file.contains("/futures-util-"),
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }

    #[test]
    fn track_caller_is_applied_on_with_context_poll() {
        block_on(async {
            let error_stream = stream::repeat(InnerSnafu.fail::<()>());
            let mut wrapped_error_stream = error_stream.with_context(|| WrappedSnafu);
            let wrapped_error = wrapped_error_stream.next().await.unwrap().unwrap_err();

            // `StreamExt::next` doesn't have `[track_caller]`, so the
            // location is inside the futures library.
            assert!(
                wrapped_error.location.file.contains("/futures-util-"),
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }

    #[test]
    fn track_caller_is_applied_on_whatever_context_poll() {
        block_on(async {
            let error_stream = stream::repeat(InnerSnafu.fail::<()>());
            let mut wrapped_error_stream = error_stream.whatever_context("bang");
            let wrapped_error: MyWhatever = wrapped_error_stream.next().await.unwrap().unwrap_err();

            // `StreamExt::next` doesn't have `[track_caller]`, so the
            // location is inside the futures library.
            assert!(
                wrapped_error.location.file.contains("/futures-util-"),
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }

    #[test]
    fn track_caller_is_applied_on_with_whatever_context_poll() {
        block_on(async {
            let error_stream = stream::repeat(InnerSnafu.fail::<()>());
            let mut wrapped_error_stream = error_stream.with_whatever_context(|_| "bang");
            let wrapped_error: MyWhatever = wrapped_error_stream.next().await.unwrap().unwrap_err();

            // `StreamExt::next` doesn't have `[track_caller]`, so the
            // location is inside the futures library.
            assert!(
                wrapped_error.location.file.contains("/futures-util-"),
                "Actual location: {}",
                wrapped_error.location,
            );
        });
    }
}
