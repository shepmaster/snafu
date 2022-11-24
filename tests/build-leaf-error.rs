use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum Error {
    Mine,
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn other_result() -> Result<i32, ()> {
    Err(())
}

fn map_result() -> Result<i32> {
    other_result().map_err(|_| MineSnafu.build())
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();

    map_result().unwrap_err();
}

#[test]
fn build_via_from_and_into() {
    let _e = Error::from(MineSnafu);
    let _e: Error = MineSnafu.into();
}
