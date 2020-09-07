use snafu::{Backtrace, Snafu};

#[test]
fn no_argument_treated_as_backtrace() {
    #[derive(Debug, Snafu)]
    struct Error {
        #[snafu(backtrace)]
        thing: Backtrace,
    }

    let _ = Context.build();
}

#[test]
fn explicit_true_treated_as_backtrace() {
    #[derive(Debug, Snafu)]
    struct Error {
        #[snafu(backtrace(true))]
        thing: Backtrace,
    }

    let _ = Context.build();
}

#[test]
fn explicit_false_not_treated_as_backtrace() {
    #[derive(Debug, Snafu)]
    struct Error {
        #[snafu(backtrace(false))]
        backtrace: i32,
    }

    let _ = Context { backtrace: 42 }.build();
}
