use snafu::prelude::*;

#[derive(Debug, Snafu)]
struct AlphaError;

#[derive(Debug, Snafu)]
struct BetaError;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(context(false))]
    Alpha { source: AlphaError },

    #[snafu(context(false))]
    Beta { source: BetaError },
}

fn alpha() -> Result<i32, AlphaError> {
    Ok(1)
}

fn beta() -> Result<i32, BetaError> {
    Ok(2)
}

fn example() -> Result<i32, Error> {
    let a = alpha()?;
    let b = beta()?;
    Ok(a * 10 + b)
}

fn check<T: std::error::Error>() {}

#[test]
fn implements_error() {
    check::<Error>();

    assert_eq!(12, example().unwrap());
}

mod with_backtraces {
    use super::*;
    use snafu::Backtrace;

    #[derive(Debug, Snafu)]
    enum Error {
        #[snafu(context(false))]
        Alpha {
            source: AlphaError,
            backtrace: Backtrace,
        },
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
        #[snafu(context(false))]
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
    #[snafu(context(false))]
    struct Error {
        #[snafu(source(from(exact)))]
        source: AlphaError,
    }

    #[test]
    fn implements_error() {
        check::<Error>();
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
    fn custom_from_implementation() {
        let _error: Error = 42.into();
    }

    // This should basically be the same as the parent module, but
    // without the explicit `from(...)` attribute.
    mod is_default_behavior {
        use super::*;

        #[derive(Debug, Snafu)]
        #[snafu(context(false))]
        struct Error {
            source: AlphaError,
        }

        #[test]
        fn implements_error() {
            check::<Error>();
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
        fn custom_from_implementation() {
            let _error: Error = 42.into();
        }
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
    enum Error {
        #[snafu(context(false))]
        Alpha {
            #[snafu(source(from(generic)))]
            source: AlphaError,
        },

        #[snafu(context(false))]
        Beta { source: BetaError },
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
