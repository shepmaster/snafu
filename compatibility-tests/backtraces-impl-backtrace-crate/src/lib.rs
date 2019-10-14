#![cfg(test)]
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
fn is_compatible_with_crate_backtrace_type() {
    fn expects_crate_type(_: &backtrace::Backtrace) {}

    let error = example().unwrap_err();
    let backtrace = ErrorCompat::backtrace(&error).unwrap();
    expects_crate_type(&backtrace);
}

#[test]
fn backtrace_contains_function_names() {
    let error = example().unwrap_err();
    let backtrace = ErrorCompat::backtrace(&error).unwrap();
    let mut names = backtrace
        .frames()
        .iter()
        .flat_map(|f| f.symbols())
        .flat_map(|s| s.name())
        .map(|n| n.to_string());
    assert!(names.any(|n| n.contains("::example::")));
}
