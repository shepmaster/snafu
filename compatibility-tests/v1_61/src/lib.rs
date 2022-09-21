#![cfg(test)]

use snafu::{prelude::*, Report};
use std::process::ExitCode;

#[test]
fn termination_returns_failure_code() {
    use std::process::Termination;

    #[derive(Debug, Snafu)]
    struct Error;

    let r = Report::from_error(Error);
    let code: ExitCode = r.report();

    assert!(
        nasty_hack_exit_code_eq(code, ExitCode::FAILURE),
        "Wanted {:?} but got {:?}",
        ExitCode::FAILURE,
        code,
    );
}

#[test]
fn procedural_macro_works_with_result_return_type() {
    #[derive(Debug, Snafu)]
    struct Error;

    #[snafu::report]
    fn mainlike_result() -> Result<(), Error> {
        Ok(())
    }

    let _: Report<Error> = mainlike_result();
}

fn nasty_hack_exit_code_eq(left: ExitCode, right: ExitCode) -> bool {
    use std::mem;

    let (left, right): (u8, u8) = unsafe {
        assert_eq!(mem::size_of::<u8>(), mem::size_of::<ExitCode>());
        (mem::transmute(left), mem::transmute(right))
    };

    left == right
}
