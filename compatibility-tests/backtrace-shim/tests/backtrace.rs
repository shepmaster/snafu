use snafu::{ensure, Backtrace, ErrorCompat, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    InvalidUser { user_id: i32, backtrace: Backtrace },
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn check_less_than(user_id: i32) -> Result<()> {
    ensure!(user_id >= 42, InvalidUser { user_id });
    Ok(())
}

fn check_greater_than(user_id: i32) -> Result<()> {
    ensure!(user_id <= 42, InvalidUser { user_id });
    Ok(())
}

fn example(user_id: i32) -> Result<()> {
    check_less_than(user_id)?;
    check_greater_than(user_id)?;

    Ok(())
}

#[test]
fn backtrace_contains_function_names() {
    let e = example(0).unwrap_err();
    let text = ErrorCompat::backtrace(&e)
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(text.contains("check_less_than"));
    assert!(text.contains("example"));
}
