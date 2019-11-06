#![cfg(test)]
#![feature(backtrace)]

use snafu::{Backtrace, ErrorCompat, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    WithBacktrace { backtrace: Backtrace },
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn example() -> Result<()> {
    WithBacktrace.fail()
}

#[test]
fn is_compatible_with_std_error_trait() {
    fn expects_std_trait<E: std::error::Error>() {}

    expects_std_trait::<Error>();
}

#[test]
fn is_compatible_with_std_backtrace_type() {
    fn expects_std_type(_: &std::backtrace::Backtrace) {}

    let error = example().unwrap_err();
    let backtrace = ErrorCompat::backtrace(&error).unwrap();
    expects_std_type(&backtrace);
}

#[test]
fn backtrace_contains_function_names() {
    let error = example().unwrap_err();
    let backtrace = ErrorCompat::backtrace(&error).unwrap();
    assert!(backtrace.to_string().contains("::example"));
}
