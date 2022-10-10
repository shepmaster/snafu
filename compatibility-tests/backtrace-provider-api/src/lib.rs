#![cfg(test)]
#![feature(error_generic_member_access, provide_any)]

use snafu::{prelude::*, IntoError};

#[test]
fn does_not_capture_a_backtrace_when_source_provides_a_backtrace() {
    #[derive(Debug, Snafu)]
    struct InnerError {
        backtrace: snafu::Backtrace,
    }

    #[derive(Debug, Snafu)]
    struct OuterError {
        source: InnerError,
        backtrace: Option<snafu::Backtrace>,
    }

    enable_backtrace_capture();
    let e = OuterSnafu.into_error(InnerSnafu.build());

    assert!(e.backtrace.is_none());
}

#[test]
fn does_capture_a_backtrace_when_source_does_not_provide_a_backtrace() {
    #[derive(Debug, Snafu)]
    struct InnerError;

    #[derive(Debug, Snafu)]
    struct OuterError {
        source: InnerError,
        backtrace: Option<snafu::Backtrace>,
    }

    enable_backtrace_capture();
    let e = OuterSnafu.into_error(InnerSnafu.build());

    assert!(e.backtrace.is_some());
}

fn enable_backtrace_capture() {
    std::env::set_var("RUST_LIB_BACKTRACE", "1");
}
