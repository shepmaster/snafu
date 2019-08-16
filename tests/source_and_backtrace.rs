extern crate snafu;

use snafu::{Backtrace, ErrorCompat, ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum InnerError {
    InnerVariant { backtrace: Backtrace },
}

#[derive(Debug, Snafu)]
enum Error {
    SourceAndBacktraceAttrs {
        // Testing source and backtrace attributes; the field should be recognized as a source,
        // and allow us to get a backtrace delegated from the source error
        #[snafu(source, backtrace)]
        cause: InnerError,
    },
}

fn example() -> Result<(), Error> {
    InnerVariant.fail().context(SourceAndBacktraceAttrs)
}

#[test]
fn delegated_backtrace_works() {
    let e = example().unwrap_err();
    ErrorCompat::backtrace(&e).unwrap();
}
