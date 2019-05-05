extern crate snafu;

use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum InnerError {
    Boom,
}

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

    ExplicitFalse {
        #[snafu(source(false))]
        source: i32,
    },
}

fn inner() -> Result<(), InnerError> {
    Ok(())
}

fn example() -> Result<(), Error> {
    inner().context(NoArgument)?;
    inner().context(ExplicitTrue)?;
    ExplicitFalse { source: 42 }.fail()?;
    Ok(())
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();
    example().unwrap_err();
}
