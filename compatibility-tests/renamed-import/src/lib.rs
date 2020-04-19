#![cfg(test)]

use a_rose::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(crate_root(a_rose))]
enum Error {
    Leaf { username: String },
    WithBacktrace { backtrace: Backtrace },
}

#[derive(Debug, Snafu)]
#[snafu(crate_root(a_rose))]
struct OpaqueError(Error);

#[test]
fn implements_std_error() {
    fn expects_std_trait<E: std::error::Error>() {}

    expects_std_trait::<Error>();
    expects_std_trait::<OpaqueError>();
}
