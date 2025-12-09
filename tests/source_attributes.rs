use snafu::prelude::*;

#[derive(Debug, Snafu)]
struct InnerError;

fn inner() -> Result<(), InnerError> {
    Ok(())
}

fn check<T: std::error::Error>() {}

mod enabling {
    use super::*;

    #[derive(Debug, Snafu)]
    enum Error {
        NoArgument {
            #[snafu(source)]
            cause: InnerError,
        },

        ExplicitTrue {
            #[snafu(source(true))]
            cause: InnerError,
        },

        FromImpliesTrue {
            #[snafu(source(from(InnerError, Box::new)))]
            cause: Box<InnerError>,
        },

        ExplicitFalse {
            #[snafu(source(false))]
            source: i32,
        },
    }

    fn example() -> Result<(), Error> {
        inner().context(NoArgumentSnafu)?;
        inner().context(ExplicitTrueSnafu)?;
        inner().context(FromImpliesTrueSnafu)?;
        ExplicitFalseSnafu { source: 42 }.fail()?;
        Ok(())
    }

    #[test]
    fn implements_error() {
        check::<Error>();
        example().unwrap_err();
    }
}

// Corresponding `from(generic)` tests are in the compile-fail suite
mod exact {
    use super::*;

    #[derive(Debug, Snafu)]
    enum EnumError {
        ExactMatch {
            #[snafu(source(from(exact)))]
            source: InnerError,
        },
    }

    #[test]
    fn enum_implements_error() {
        fn example() -> Result<(), EnumError> {
            inner().context(ExactMatchSnafu)?;
            Ok(())
        }

        check::<EnumError>();
        example().unwrap();
    }

    #[derive(Debug, Snafu)]
    struct StructError {
        #[snafu(source(from(exact)))]
        source: InnerError,
    }

    #[test]
    fn struct_implements_error() {
        fn example() -> Result<(), StructError> {
            inner().context(StructSnafu)?;
            Ok(())
        }

        check::<StructError>();
        example().unwrap();
    }
}

mod transformation {
    use super::*;
    use std::io;

    #[derive(Debug, Snafu)]
    enum Error {
        TransformationViaClosure {
            #[snafu(source(from(InnerError, |e| io::Error::new(io::ErrorKind::InvalidData, e))))]
            source: io::Error,
        },

        TransformationViaFunction {
            #[snafu(source(from(InnerError, into_io)))]
            source: io::Error,
        },

        TransformationToTraitObject {
            #[snafu(source(from(InnerError, Box::new)))]
            source: Box<dyn std::error::Error>,
        },
    }

    fn into_io(e: InnerError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, e)
    }

    fn example() -> Result<(), Error> {
        inner().context(TransformationViaClosureSnafu)?;
        inner().context(TransformationViaFunctionSnafu)?;
        inner().context(TransformationToTraitObjectSnafu)?;
        Ok(())
    }

    #[test]
    fn implements_error() {
        check::<Error>();
        example().unwrap();
    }

    #[derive(Debug, Snafu)]
    #[snafu(source(from(Error, Box::new)))]
    struct ApiError(Box<Error>);

    fn api_example() -> Result<(), ApiError> {
        example()?;
        Ok(())
    }

    #[test]
    fn api_implements_error() {
        check::<ApiError>();
        api_example().unwrap();
    }
}
