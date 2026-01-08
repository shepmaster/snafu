#![cfg(test)]
#![feature(error_generic_member_access)]

use snafu::{prelude::*, Report};
use std::process::ExitCode;

#[test]
fn provided_exit_code_is_returned() {
    use std::process::Termination;

    #[derive(Debug, Snafu)]
    enum TwoKindError {
        #[snafu(provide(ExitCode => ExitCode::from(2)))]
        Mild,
        #[snafu(provide(ExitCode => ExitCode::from(3)))]
        Extreme,
    }

    let mild = Report::from_error(MildSnafu.build()).report();
    let expected_mild = ExitCode::from(2);

    assert_eq!(mild, expected_mild);

    let extreme = Report::from_error(ExtremeSnafu.build()).report();
    let expected_extreme = ExitCode::from(3);

    assert_eq!(extreme, expected_extreme);
}

#[test]
fn provided_backtrace_is_printed() {
    #[derive(Debug, Snafu)]
    struct Error {
        backtrace: snafu::Backtrace,
    }

    let r = Report::from_error(Snafu.build());
    let msg = r.to_string();

    let this_function = "::provided_backtrace_is_printed";
    assert!(
        msg.contains(this_function),
        "Expected {msg:?} to contain {this_function:?}"
    );
}
