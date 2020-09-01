use snafu::{Backtrace, ErrorCompat, Snafu};

#[test]
fn can_include_a_backtrace() {
    #[derive(Debug, Snafu)]
    struct Error {
        backtrace: Backtrace,
    }

    let e = Context.build();
    let backtrace = ErrorCompat::backtrace(&e);
    assert!(backtrace.is_some());
}
