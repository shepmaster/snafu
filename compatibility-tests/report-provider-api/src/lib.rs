#![cfg(test)]
#![feature(error_generic_member_access)]

use snafu::{prelude::*, Report};
use std::process::ExitCode;

#[test]
#[ignore] // https://github.com/rust-lang/rust/pull/114973
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

    assert!(
        nasty_hack_exit_code_eq(mild, expected_mild),
        "Wanted {:?} but got {:?}",
        expected_mild,
        mild,
    );

    let extreme = Report::from_error(ExtremeSnafu.build()).report();
    let expected_extreme = ExitCode::from(3);

    assert!(
        nasty_hack_exit_code_eq(extreme, expected_extreme),
        "Wanted {:?} but got {:?}",
        expected_extreme,
        extreme,
    );
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

fn nasty_hack_exit_code_eq(left: ExitCode, right: ExitCode) -> bool {
    use std::mem;

    let (left, right): (u8, u8) = unsafe {
        assert_eq!(mem::size_of::<u8>(), mem::size_of::<ExitCode>());
        (mem::transmute(left), mem::transmute(right))
    };

    left == right
}
