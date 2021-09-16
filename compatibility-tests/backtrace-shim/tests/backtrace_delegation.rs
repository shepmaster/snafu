use snafu::{prelude::*, ErrorCompat};

mod house {
    use snafu::{prelude::*, Backtrace};

    #[derive(Debug, Snafu)]
    pub enum Error {
        Fatal { backtrace: Backtrace },
    }

    pub fn answer_telephone() -> Result<(), Error> {
        FatalSnafu.fail()
    }
}

#[derive(Debug, Snafu)]
enum Error {
    MovieTrope {
        #[snafu(backtrace)]
        source: house::Error,
    },
    SourceAndBacktraceAttrs {
        // Testing source and backtrace attributes; the field should be recognized as a source,
        // and allow us to get a backtrace delegated from the source error
        #[snafu(source, backtrace)]
        cause: house::Error,
    },
}

fn delegate_example() -> Result<(), Error> {
    house::answer_telephone().context(MovieTropeSnafu)?;

    Ok(())
}

#[test]
fn backtrace_comes_from_delegated_error() {
    let e = delegate_example().unwrap_err();
    let text = ErrorCompat::backtrace(&e)
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(
        text.contains("answer_telephone"),
        "{:?} does not contain `answer_telephone`",
        text
    );
}

fn delegate_and_rename_example() -> Result<(), Error> {
    house::answer_telephone().context(SourceAndBacktraceAttrsSnafu)
}

#[test]
fn backtrace_comes_from_renamed_delegated_error() {
    let e = delegate_and_rename_example().unwrap_err();
    let text = ErrorCompat::backtrace(&e)
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(
        text.contains("answer_telephone"),
        "{:?} does not contain `answer_telephone`",
        text
    );
}
