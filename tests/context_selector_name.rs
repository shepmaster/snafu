use snafu::Snafu;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(context(suffix(Moo)))]
    Alpha,

    TrimsWhenEndingInError,
}

fn alpha_usage() -> Result<(), Error> {
    AlphaMoo.fail()
}

fn trimmming_usage() -> Result<(), Error> {
    TrimsWhenEndingIn.fail()
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();

    alpha_usage().unwrap_err();
    trimmming_usage().unwrap_err();
}
