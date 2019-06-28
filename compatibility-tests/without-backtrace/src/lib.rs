extern crate snafu;

use snafu::Snafu;

#[derive(Debug, Snafu)]
struct PublicError(Error);

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display = r#"("User ID {} is invalid", user_id)"#)]
    InvalidUser { user_id: i32 },
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();
    check::<PublicError>();
}
