use snafu::{Backtrace, ErrorCompat, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    BacktraceSometimes { backtrace: Option<Backtrace> },
}

#[test]
fn optional_backtrace_is_present_with_environment_variable() {
    std::env::set_var("RUST_LIB_BACKTRACE", "1");
    let sometimes = BacktraceSometimes.build();
    assert!(ErrorCompat::backtrace(&sometimes).is_some());
}
