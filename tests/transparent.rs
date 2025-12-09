use snafu::prelude::*;
use std::error::Error;

#[derive(Debug, Snafu)]
struct AlphaError;

#[derive(Debug, Snafu)]
struct BetaError {
    source: AlphaError,
}

#[derive(Debug, Snafu)]
enum ErrorTest {
    #[snafu(transparent)]
    Alpha { source: AlphaError },

    #[snafu(transparent)]
    Beta { source: BetaError },
}

fn alpha() -> Result<i32, AlphaError> {
    AlphaSnafu.fail()
}

fn beta() -> Result<i32, BetaError> {
    let a = alpha().context(BetaSnafu)?;
    Ok(a + 2)
}

fn example() -> Result<i32, ErrorTest> {
    let a = 2;
    let b = beta()?;
    Ok(a * 10 + b)
}

fn check<T: std::error::Error>() {}

#[test]
fn implements_error() {
    check::<ErrorTest>();

    let error = example().unwrap_err();
    // skipping `ErrorTest`, directly going to `BetaError`'s source
    assert_eq!(error.source().unwrap().to_string(), "AlphaError");
}

#[test]
fn implements_display() {
    let error = example().unwrap_err();
    // no `Beta: ` prefix
    assert_eq!(error.to_string(), "BetaError");
}

mod with_backtraces {
    use super::*;
    use snafu::Backtrace;

    #[derive(Debug, Snafu)]
    #[snafu(transparent)]
    struct Error {
        source: AlphaError,
        backtrace: Backtrace,
    }

    #[test]
    fn implements_error() {
        check::<Error>();
    }
}

mod with_bounds {
    use super::*;
    use std::fmt::{Debug, Display};

    #[derive(Debug, Snafu)]
    enum GenericError<T, U = i32> {
        _Something { things: T, other: U },
    }

    #[derive(Debug, Snafu)]
    enum Error<T: 'static>
    where
        T: Debug + Display + Copy,
    {
        #[snafu(transparent)]
        Generic { source: GenericError<T> },
    }

    #[test]
    fn implements_error() {
        check::<Error<i32>>();
    }
}

mod with_exact_source {
    use super::*;

    #[derive(Debug, Snafu)]
    #[snafu(transparent)]
    struct Error {
        #[snafu(source(from(exact)))]
        source: AlphaError,
    }

    trait LocalTrait {}
    impl LocalTrait for i32 {}

    impl<T> From<T> for Error
    where
        T: LocalTrait,
    {
        fn from(_: T) -> Self {
            Error { source: AlphaError }
        }
    }

    #[test]
    fn implements_error() {
        check::<Error>();
    }

    #[test]
    fn custom_from_implementation() {
        let _error: Error = 42.into();
    }
}

mod with_generic_source {
    use super::*;

    struct NotAlpha;

    impl From<NotAlpha> for AlphaError {
        fn from(_: NotAlpha) -> Self {
            AlphaError
        }
    }

    fn convertable_to_alpha() -> Result<(), NotAlpha> {
        Ok(())
    }

    #[derive(Debug, Snafu)]
    #[snafu(transparent)]
    struct Error {
        #[snafu(source(from(generic)))]
        source: AlphaError,
    }

    #[test]
    fn implements_error() {
        check::<Error>();
    }

    #[test]
    fn converted_to_alpha() {
        fn converts_to_alpha() -> Result<(), Error> {
            convertable_to_alpha()?;
            Ok(())
        }

        converts_to_alpha().unwrap()
    }
}
