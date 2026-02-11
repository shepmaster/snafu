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

#[test]
fn provided_location_is_printed() {
    use snafu::IntoError;

    #[derive(Debug, Snafu)]
    struct InnerError {
        // The shorthand provides a `&&'static Location<'static>`
        #[snafu(implicit, provide)]
        location: snafu::Location,
    }

    #[derive(Debug, Snafu)]
    // This longhand provides a `&Location<'static>`
    #[snafu(provide(ref, core::panic::Location => location))]
    struct MiddleError {
        source: InnerError,
        #[snafu(implicit)]
        location: snafu::Location,
    }

    #[derive(Debug, Snafu)]
    // This longhand provides a `&'static Location<'static>`
    #[snafu(provide(&'static core::panic::Location => location))]
    struct OuterError {
        source: MiddleError,
        #[snafu(implicit)]
        location: snafu::Location,
    }

    let a = InnerSnafu.build();
    let l_a = a.location.to_string();

    let b = MiddleSnafu.into_error(a);
    let l_b = b.location.to_string();

    let c = OuterSnafu.into_error(b);
    let l_c = c.location.to_string();

    let r = Report::from_error(c);
    let msg = r.to_string();

    assert!(msg.contains(&l_a), "Expected {msg:?} to contain {l_a}");
    assert!(msg.contains(&l_b), "Expected {msg:?} to contain {l_b}");
    assert!(msg.contains(&l_c), "Expected {msg:?} to contain {l_c}");
}
