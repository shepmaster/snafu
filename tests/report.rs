use snafu::{prelude::*, IntoError, Report};

#[test]
fn includes_the_error_display_text() {
    #[derive(Debug, Snafu)]
    #[snafu(display("This is my Display text!"))]
    struct Error;

    let r = Report::from_error(Error);
    let msg = r.to_string();

    let expected = "This is my Display text!";
    assert!(
        msg.contains(expected),
        "Expected {:?} to include {:?}",
        msg,
        expected,
    );
}

#[test]
fn includes_the_source_display_text() {
    #[derive(Debug, Snafu)]
    #[snafu(display("This is my inner Display"))]
    struct InnerError;

    #[derive(Debug, Snafu)]
    #[snafu(display("This is my outer Display"))]
    struct OuterError {
        source: InnerError,
    }

    let e = OuterSnafu.into_error(InnerError);
    let r = Report::from_error(e);
    let msg = r.to_string();

    let expected = "This is my inner Display";
    assert!(
        msg.contains(expected),
        "Expected {:?} to include {:?}",
        msg,
        expected,
    );
}

#[test]
fn debug_and_display_are_the_same() {
    #[derive(Debug, Snafu)]
    #[snafu(display("This is my inner Display"))]
    struct InnerError;

    #[derive(Debug, Snafu)]
    #[snafu(display("This is my outer Display"))]
    struct OuterError {
        source: InnerError,
    }

    let e = OuterSnafu.into_error(InnerError);
    let r = Report::from_error(e);

    let display = format!("{}", r);
    let debug = format!("{:?}", r);

    assert_eq!(display, debug);
}

#[test]
fn procedural_macro_works_with_result_return_type() {
    #[derive(Debug, Snafu)]
    struct Error;

    #[snafu::report]
    fn mainlike_result() -> Result<(), Error> {
        Ok(())
    }

    let _: Result<(), Report<Error>> = mainlike_result();
}

#[derive(Debug, Snafu)]
struct TestFunctionError;

#[test]
#[snafu::report]
fn procedural_macro_works_with_test_functions() -> Result<(), TestFunctionError> {
    Ok(())
}
