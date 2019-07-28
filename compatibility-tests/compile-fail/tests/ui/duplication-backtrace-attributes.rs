use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
enum EnumError {
    AVariant {
        // Second attribute should be marked as duplicate
        #[snafu(backtrace)]
        #[snafu(backtrace)]
        my_backtrace: Backtrace,
    },
}

fn main() {}
