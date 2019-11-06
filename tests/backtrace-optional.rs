use snafu::{Backtrace, ErrorCompat, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    BacktraceAlways { backtrace: Backtrace },
    BacktraceSometimes { backtrace: Option<Backtrace> },
}

#[test]
fn bare_backtrace_is_always_present() {
    let always = BacktraceAlways.fail::<()>();
    assert!(ErrorCompat::backtrace(&always.unwrap_err()).is_some());
}

#[test]
fn optional_backtrace_is_not_present_without_environment_variable() {
    let sometimes = BacktraceSometimes.fail::<()>();
    assert!(ErrorCompat::backtrace(&sometimes.unwrap_err()).is_none());
}
