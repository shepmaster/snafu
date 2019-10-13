use snafu::{Backtrace, ErrorCompat, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    BacktraceSometimes { backtrace: Option<Backtrace> },
}

#[test]
fn optional_backtrace_is_present_with_environment_variable() {
    std::env::set_var("RUST_LIB_BACKTRACE", "1");
    let sometimes = BacktraceSometimes.fail::<()>();
    assert!(ErrorCompat::backtrace(&sometimes.unwrap_err()).is_some());
}
