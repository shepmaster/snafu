// This test asserts that `ResultExt::eager_context` can be used even
// when `try!` or `?` is not used.

use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum InnerError {
    Alpha,
}

#[derive(Debug, Snafu)]
enum OuterError {
    Beta { source: InnerError },
}

fn function_1() -> Result<i32, InnerError> {
    Alpha.fail()
}

fn function_2() -> Result<bool, OuterError> {
    function_1().map(|v| v < 42).context(Beta)
}

#[test]
fn can_be_used_without_question_mark() {
    function_2().unwrap_err();
}
