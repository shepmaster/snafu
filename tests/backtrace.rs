extern crate snafu;

use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};

type AnotherError = Box<std::error::Error>;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display = r#"("Invalid user {}:\n{}", user_id, backtrace)"#)]
    InvalidUser { user_id: i32, backtrace: Backtrace },
    WithSource {
        source: AnotherError,
        backtrace: Backtrace,
    },
    WithSourceAndOtherInfo {
        user_id: i32,
        source: AnotherError,
        backtrace: Backtrace,
    },
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
fn has_a_backtrace() {
    let e = example(0).unwrap_err();
    let text = ErrorCompat::backtrace(&e)
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(text.contains("check_less_than"));
}

#[test]
fn display_can_access_backtrace() {
    let e = example(0).unwrap_err();
    let text = e.to_string();
    assert!(text.contains("check_less_than"));
}

fn trigger() -> Result<(), AnotherError> {
    Err("boom".into())
}

#[test]
fn errors_with_sources_can_have_backtraces() {
    let _e: Error = trigger().context(WithSource).unwrap_err();
}

#[test]
fn errors_with_sources_and_other_info_can_have_backtraces() {
    let _e: Error = trigger()
        .context(WithSourceAndOtherInfo { user_id: 42 })
        .unwrap_err();
}
