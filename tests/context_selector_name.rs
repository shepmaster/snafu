use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(context(suffix(Moo)))]
    Alpha,

    TrimsWhenEndingInError,

    #[snafu(context(suffix(false)))]
    CanOptOutOfSuffix,
}

fn alpha_usage() -> Result<(), Error> {
    AlphaMoo.fail()
}

fn trimming_usage() -> Result<(), Error> {
    TrimsWhenEndingInSnafu.fail()
}

fn no_suffix_usage() -> Result<(), Error> {
    CanOptOutOfSuffix.fail()
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();

    alpha_usage().unwrap_err();
    trimming_usage().unwrap_err();
    no_suffix_usage().unwrap_err();
}
