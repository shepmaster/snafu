#![cfg(test)]

use a_rose::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(crate_root(a_rose))]
enum EnumError {
    _Leaf { username: String },
    _WithBacktrace { backtrace: Backtrace },
}

#[derive(Debug, Snafu)]
#[snafu(crate_root(a_rose))]
struct OpaqueError(EnumError);

#[derive(Debug, Snafu)]
#[snafu(crate_root(a_rose))]
struct StructError;

#[test]
fn implements_std_error() {
    fn expects_std_trait<E: std::error::Error>() {}

    expects_std_trait::<EnumError>();
    expects_std_trait::<OpaqueError>();
    expects_std_trait::<StructError>();
}
