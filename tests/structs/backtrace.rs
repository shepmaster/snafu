use snafu::{Backtrace, ErrorCompat, IntoError, Snafu};

#[test]
fn can_include_a_backtrace_in_leaf() {
    #[derive(Debug, Snafu)]
    struct Error {
        backtrace: Backtrace,
    }

    let e = Context.build();
    let backtrace = ErrorCompat::backtrace(&e);
    assert!(backtrace.is_some());
}

#[test]
fn can_include_a_backtrace_with_source() {
    #[derive(Debug, Snafu)]
    struct InnerError;

    #[derive(Debug, Snafu)]
    struct Error {
        source: InnerError,
        backtrace: Backtrace,
    }

    let i = InnerContext.build();
    let e = Context.into_error(i);
    let backtrace = ErrorCompat::backtrace(&e);
    assert!(backtrace.is_some());
}

#[test]
fn can_include_a_backtrace_with_no_context() {
    #[derive(Debug, Snafu)]
    struct InnerError;

    #[derive(Debug, Snafu)]
    #[snafu(context(false))]
    struct Error {
        source: InnerError,
        backtrace: Backtrace,
    }

    let i = InnerContext.build();
    let e = Error::from(i);
    let backtrace = ErrorCompat::backtrace(&e);
    assert!(backtrace.is_some());
}
