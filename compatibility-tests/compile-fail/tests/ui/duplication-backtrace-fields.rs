use snafu::{prelude::*, Backtrace};

#[derive(Debug, Snafu)]
enum EnumError {
    AVariant {
        // First backtrace, legitimate
        backtrace: Backtrace,

        // Trying to declare another field as backtrace, should be a duplicate
        #[snafu(backtrace)]
        my_backtrace: Backtrace,
    },

    AnotherVariant {
        // First backtrace, legitimate
        #[snafu(backtrace)]
        my_backtrace: Backtrace,

        // Trying to declare another field as backtrace, should be a duplicate
        backtrace: Backtrace,
    },
}

fn main() {}
