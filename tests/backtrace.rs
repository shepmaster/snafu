use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};

type AnotherError = Box<dyn std::error::Error>;

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

fn example(user_id: i32) -> Result<()> {
    ensure!(user_id >= 42, InvalidUser { user_id });
    Ok(())
}

#[test]
fn display_can_access_backtrace() {
    let e = example(0).unwrap_err();
    let text = e.to_string();
    assert!(
        text.contains("disabled backtrace"),
        "{:?} does not contain expected text",
        text
    );
}

fn trigger() -> Result<(), AnotherError> {
    Err("boom".into())
}

#[test]
fn errors_with_sources_can_have_backtraces() {
    let e = trigger().context(WithSource).unwrap_err();
    let backtrace = ErrorCompat::backtrace(&e).unwrap();
    assert!(backtrace.to_string().contains("disabled backtrace"));
}

#[test]
fn errors_with_sources_and_other_info_can_have_backtraces() {
    let e = trigger()
        .context(WithSourceAndOtherInfo { user_id: 42 })
        .unwrap_err();
    let backtrace = ErrorCompat::backtrace(&e).unwrap();
    assert!(backtrace.to_string().contains("disabled backtrace"));
}
