use snafu::{Backtrace, ErrorCompat, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    BacktraceAlways { backtrace: Backtrace },
    BacktraceSometimes { backtrace: Option<Backtrace> },
}

#[test]
fn bare_backtrace_is_always_present() {
    let always = BacktraceAlways.build();
    assert!(ErrorCompat::backtrace(&always).is_some());
}

#[test]
fn optional_backtrace_is_not_present_without_environment_variable() {
    let sometimes = BacktraceSometimes.build();
    assert!(ErrorCompat::backtrace(&sometimes).is_none());
}
